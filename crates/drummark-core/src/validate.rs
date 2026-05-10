use crate::resolve::is_valid_track;

/// Check if a modifier is legal on a given track.
/// Returns an error message if illegal, None if legal.
pub fn validate_modifier_legality(modifier: &str, track: &str) -> Option<String> {
    let legal_tracks = match modifier {
        "accent" => ALL_TRACKS,
        "open" => &["HH"][..],
        "half-open" => &["HH"][..],
        "close" => &["HH", "HF"][..],
        "choke" => &["RC", "RC2", "C", "C2", "SPL", "CHN"][..],
        "bell" => &["RC", "RC2"][..],
        "rim" => &["SD"][..],
        "cross" => &["SD"][..],
        "flam" => &["SD", "T1", "T2", "T3", "T4"][..],
        "ghost" => &["SD", "HH", "T1", "T2", "T3", "T4"][..],
        "drag" => &["SD", "HH", "T1", "T2", "T3", "T4", "RC", "RC2"][..],
        "roll" => &["SD", "HH", "T1", "T2", "T3", "T4", "RC", "RC2", "BD", "BD2"][..],
        "dead" => &["SD", "HH", "T1", "T2", "T3", "T4", "BD", "BD2"][..],
        _ => return Some(format!("unknown modifier: {modifier}")),
    };

    if legal_tracks.contains(&track) {
        None
    } else {
        Some(format!(
            "modifier \"{modifier}\" is not legal on track \"{track}\""
        ))
    }
}

/// Validate that grouping values sum to the time signature's beats,
/// and that each value produces integer boundaries at the given divisions.
/// Returns an error message if invalid.
pub fn validate_grouping(
    grouping: &[u32],
    beats: u32,
    beat_unit: u32,
    divisions: u32,
) -> Option<String> {
    let sum: u32 = grouping.iter().sum();
    if sum != beats {
        return Some(format!(
            "grouping sum {sum} must equal time numerator {beats}"
        ));
    }

    for (i, &g) in grouping.iter().enumerate() {
        let boundary = g * divisions;
        if boundary % beats != 0 {
            return Some(format!(
                "grouping segment {} value {} does not fall on integer slot boundary",
                i + 1, g
            ));
        }
        let _ = beat_unit; // reserved for future boundary checks
    }
    None
}

/// Validate a group (tuplet) token's ratio.
/// Returns an error message if invalid.
pub fn validate_group_token(
    n: u32,
    item_count: usize,
) -> Option<String> {
    const VALID_RATIOS: &[(u32, u32)] = &[
        (2, 1), (3, 1), (4, 1),
        (3, 2), (4, 2), (5, 4), (6, 4), (7, 4),
    ];

    let actual = item_count as u32;

    let valid = VALID_RATIOS.contains(&(n, actual));
    let also_valid = n == 0 || n == actual; // no explicit n, or n == count

    if !valid && !also_valid {
        Some(format!(
            "invalid group ratio {n}:{actual} (valid ratios: {:?})",
            VALID_RATIOS
        ))
    } else {
        None
    }
}

const ALL_TRACKS: &[&str] = &[
    "HH", "HF", "SD", "BD", "T1", "T2", "T3", "RC", "C", "ST",
    "BD2", "T4", "RC2", "C2", "SPL", "CHN", "CB", "WB", "CL",
];

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_legal() {
        assert!(validate_modifier_legality("accent", "HH").is_none());
        assert!(validate_modifier_legality("open", "HH").is_none());
        assert!(validate_modifier_legality("cross", "SD").is_none());
    }

    #[test]
    fn test_modifier_illegal() {
        assert!(validate_modifier_legality("open", "SD").is_some());
        assert!(validate_modifier_legality("rim", "HH").is_some());
        assert!(validate_modifier_legality("choke", "HH").is_some());
    }

    #[test]
    fn test_grouping_valid() {
        assert!(validate_grouping(&[2, 2], 4, 4, 16).is_none());
        assert!(validate_grouping(&[3, 3], 6, 8, 12).is_none());
    }

    #[test]
    fn test_grouping_invalid_sum() {
        assert!(validate_grouping(&[3, 2], 7, 8, 14).is_some());
    }

    #[test]
    fn test_group_token_valid() {
        assert!(validate_group_token(2, 1).is_none());
        assert!(validate_group_token(3, 2).is_none());
        assert!(validate_group_token(0, 3).is_none()); // n=0 → n==count fallback
    }

    #[test]
    fn test_group_token_invalid() {
        assert!(validate_group_token(5, 3).is_some());
    }
}
