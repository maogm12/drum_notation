use std::cmp::Ordering;

/// Rational number with u64 numerator and denominator.
/// Always stored in simplified form (gcd-reduced, denominator > 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl Fraction {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        assert!(denominator > 0, "denominator must be positive");
        Self { numerator, denominator }.simplify()
    }

    pub fn zero() -> Self {
        Self { numerator: 0, denominator: 1 }
    }

    pub fn one() -> Self {
        Self { numerator: 1, denominator: 1 }
    }

    // ── Arithmetic ──────────────────────────────────────────

    pub fn simplify(self) -> Self {
        let g = gcd(self.numerator, self.denominator);
        let n = self.numerator / g;
        let d = self.denominator / g;
        Self { numerator: n, denominator: d }
    }

    pub fn add(self, other: Self) -> Self {
        // Use u128 intermediates to avoid overflow
        let n = (self.numerator as u128) * (other.denominator as u128)
            + (other.numerator as u128) * (self.denominator as u128);
        let d = (self.denominator as u128) * (other.denominator as u128);
        Fraction {
            numerator: safe_downcast(n),
            denominator: safe_downcast(d),
        }
        .simplify()
    }

    pub fn subtract(self, other: Self) -> Self {
        let left = (self.numerator as u128) * (other.denominator as u128);
        let right = (other.numerator as u128) * (self.denominator as u128);
        if left < right {
            return Fraction::zero();
        }
        let n = left - right;
        let d = (self.denominator as u128) * (other.denominator as u128);
        Fraction {
            numerator: safe_downcast(n),
            denominator: safe_downcast(d),
        }
        .simplify()
    }

    pub fn multiply(self, other: Self) -> Self {
        Fraction {
            numerator: safe_downcast(
                (self.numerator as u128) * (other.numerator as u128),
            ),
            denominator: safe_downcast(
                (self.denominator as u128) * (other.denominator as u128),
            ),
        }
        .simplify()
    }

    pub fn divide(self, other: Self) -> Self {
        Fraction {
            numerator: safe_downcast(
                (self.numerator as u128) * (other.denominator as u128),
            ),
            denominator: safe_downcast(
                (self.denominator as u128) * (other.numerator as u128),
            ),
        }
        .simplify()
    }

    pub fn multiply_scalar(self, scalar: u32) -> Self {
        self.multiply(Fraction {
            numerator: scalar as u64,
            denominator: 1,
        })
    }

    pub fn divide_scalar(self, scalar: u32) -> Self {
        self.divide(Fraction {
            numerator: scalar as u64,
            denominator: 1,
        })
    }

    // ── Comparisons ─────────────────────────────────────────

    pub fn compare(self, other: Self) -> Ordering {
        let left = (self.numerator as u128) * (other.denominator as u128);
        let right = (other.numerator as u128) * (self.denominator as u128);
        left.cmp(&right)
    }

    pub fn is_zero(self) -> bool {
        self.numerator == 0
    }

    // ── Conversion ──────────────────────────────────────────

    pub fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Convert to a slot count for the given time signature and divisions.
    /// slot = fraction * (beat_unit * divisions / beats)
    pub fn to_slot_count(
        self,
        divisions: u32,
        beat_unit: u32,
        beats: u32,
    ) -> f64 {
        let slots_per_whole = divisions as f64;
        let whole_ratio = self.numerator as f64 / self.denominator as f64;
        // fraction is relative to a whole note? No, in DrumMark, fractions are
        // relative to a quarter note when note value is 1/4 etc.
        // This is a simplified approximation — the caller adjusts.
        whole_ratio * slots_per_whole * (beat_unit as f64 / beats as f64)
    }
}

// ── Free functions ───────────────────────────────────────────────

pub fn gcd(a: u64, b: u64) -> u64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let next = x % y;
        x = y;
        y = next;
    }
    if x == 0 { 1 } else { x }
}

pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b)).saturating_mul(b)
    }
}

pub fn fractions_equal(left: Fraction, right: Fraction) -> bool {
    let a = left.simplify();
    let b = right.simplify();
    a.numerator == b.numerator && a.denominator == b.denominator
}

/// Check if a fraction's denominator exceeds the IEEE 754 53-bit mantissa
/// limit, which would cause precision loss when serialized to JSON.
/// The TS equivalent checks `denominator > 2^53`.
pub fn exceeds_exact_duration_range(denominator: u64) -> bool {
    denominator > (1u64 << 53)
}

// ── Helpers ──────────────────────────────────────────────────────

fn safe_downcast(n: u128) -> u64 {
    if n > u64::MAX as u128 {
        // Saturate — music-domain values should never hit this
        u64::MAX
    } else {
        n as u64
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify() {
        assert_eq!(Fraction::new(2, 4), Fraction::new(1, 2));
        assert_eq!(Fraction::new(6, 8), Fraction::new(3, 4));
        assert_eq!(Fraction::new(0, 5), Fraction::new(0, 1));
    }

    #[test]
    fn test_add() {
        let a = Fraction::new(1, 4);
        let b = Fraction::new(1, 4);
        assert_eq!(a.add(b), Fraction::new(1, 2));
    }

    #[test]
    fn test_multiply() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 2);
        assert_eq!(a.multiply(b), Fraction::new(1, 4));
    }

    #[test]
    fn test_divide() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 4);
        assert_eq!(a.divide(b), Fraction::new(2, 1));
    }

    #[test]
    fn test_subtract() {
        let a = Fraction::new(3, 4);
        let b = Fraction::new(1, 4);
        assert_eq!(a.subtract(b), Fraction::new(1, 2));
    }

    #[test]
    fn test_subtract_underflow() {
        let a = Fraction::new(1, 4);
        let b = Fraction::new(3, 4);
        assert_eq!(a.subtract(b), Fraction::zero());
    }

    #[test]
    fn test_compare() {
        assert!(Fraction::new(1, 2).compare(Fraction::new(1, 4)) == Ordering::Greater);
        assert!(Fraction::new(1, 4).compare(Fraction::new(1, 2)) == Ordering::Less);
        assert!(Fraction::new(2, 4).compare(Fraction::new(1, 2)) == Ordering::Equal);
    }

    #[test]
    fn test_fractions_equal() {
        assert!(fractions_equal(Fraction::new(2, 4), Fraction::new(1, 2)));
        assert!(!fractions_equal(Fraction::new(1, 3), Fraction::new(1, 2)));
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(7, 13), 1);
        assert_eq!(gcd(0, 5), 5);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(7, 13), 91);
    }

    #[test]
    fn test_exceeds_range() {
        assert!(!exceeds_exact_duration_range(4));
        assert!(!exceeds_exact_duration_range(1u64 << 53));
        assert!(exceeds_exact_duration_range((1u64 << 53) + 1));
    }

    #[test]
    fn test_calculate_token_weight_simple() {
        // No dots, no stars, no halves → weight = 1
        let f = calculate_token_weight_as_fraction(0, 0, 0, None);
        assert_eq!(f, Fraction::new(1, 1));
    }

    #[test]
    fn test_calculate_token_weight_dots() {
        // 1 dot → weight = 3/2 (1.5x)
        let f = calculate_token_weight_as_fraction(1, 0, 0, None);
        assert_eq!(f, Fraction::new(3, 2));
    }

    #[test]
    fn test_calculate_token_weight_double_dot() {
        // 2 dots → weight = 7/4 (1.75x)
        let f = calculate_token_weight_as_fraction(2, 0, 0, None);
        assert_eq!(f, Fraction::new(7, 4));
    }

    #[test]
    fn test_calculate_token_weight_stars() {
        // 1 star → weight = 2
        let f = calculate_token_weight_as_fraction(0, 1, 0, None);
        assert_eq!(f, Fraction::new(2, 1));
    }

    #[test]
    fn test_calculate_token_weight_halves() {
        // 1 half → weight = 1/2
        let f = calculate_token_weight_as_fraction(0, 0, 1, None);
        assert_eq!(f, Fraction::new(1, 2));
    }

    #[test]
    fn test_calculate_token_weight_stars_and_halves_cancel() {
        // 1 star, 1 half → weight = 1
        let f = calculate_token_weight_as_fraction(0, 1, 1, None);
        assert_eq!(f, Fraction::new(1, 1));
    }

    #[test]
    fn test_calculate_token_weight_tuplet_span() {
        // tuplet span = 2 → weight = 2
        let f = calculate_token_weight_as_fraction(0, 0, 0, Some(2));
        assert_eq!(f, Fraction::new(2, 1));
    }
}

/// Calculate token weight as a Fraction.
///
/// Weight formula: weight = base * (2 - 0.5^dots) * (2^stars) / (2^halves)
/// dots=1 → 1.5 = 3/2
/// dots=2 → 1.75 = 7/4
/// numerator = 2^(dots+1) - 1, denominator = 2^dots
///
/// Tuplet span overrides base: tuplet_span replaces the 1 base weight.
pub fn calculate_token_weight_as_fraction(
    dots: u32,
    stars: u32,
    halves: u32,
    tuplet_span: Option<u32>,
) -> Fraction {
    let base = match tuplet_span {
        Some(s) if s > 0 => s as u64,
        _ => 1,
    };

    // Dot weight = (2^(dots+1) - 1) / 2^dots
    let dot_denom = 1u64 << dots; // 2^dots
    let dot_num = (1u64 << (dots + 1))
        .saturating_sub(1);
    let dot_weight = Fraction::new(dot_num, dot_denom);

    // Net stars/halves
    let net = stars as i32 - halves as i32;
    let (star_mul, half_div) = if net > 0 {
        (1u64 << (net as u32), 1u64)
    } else {
        (1u64, 1u64 << ((-net) as u32))
    };

    Fraction::new(dot_num.saturating_mul(base).saturating_mul(star_mul), dot_denom.saturating_mul(half_div))
        .simplify()
}
