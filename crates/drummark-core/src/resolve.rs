// ── Track family sets ───────────────────────────────────────────

const CYMBAL_TRACKS: &[&str] = &[
    "HH", "RC", "RC2", "C", "C2", "SPL", "CHN",
];

const DRUM_TRACKS: &[&str] = &[
    "SD", "BD", "BD2", "T1", "T2", "T3", "T4", "ST",
];

const PEDAL_TRACKS: &[&str] = &["HF"];

const PERCUSSION_TRACKS: &[&str] = &["CB", "WB", "CL"];

const ALL_TRACKS: &[&str] = &[
    "HH", "HF", "SD", "BD", "T1", "T2", "T3", "RC", "C", "ST",
    "BD2", "T4", "RC2", "C2", "SPL", "CHN", "CB", "WB", "CL",
];

/// Track family classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackFamily {
    Cymbal,
    Drum,
    Pedal,
    Percussion,
    Auxiliary,
}

/// Voice assignment: BD, BD2, HF → voice 2; everything else → voice 1.
pub fn voice_for_track(track: &str) -> u8 {
    match track {
        "BD" | "BD2" | "HF" => 2,
        _ => 1,
    }
}

/// Returns the track family for a given track name.
pub fn get_track_family(track: &str) -> TrackFamily {
    if CYMBAL_TRACKS.contains(&track) { return TrackFamily::Cymbal; }
    if DRUM_TRACKS.contains(&track)    { return TrackFamily::Drum; }
    if PEDAL_TRACKS.contains(&track)   { return TrackFamily::Pedal; }
    if PERCUSSION_TRACKS.contains(&track) { return TrackFamily::Percussion; }
    TrackFamily::Auxiliary
}

/// Check if a string is a valid track name.
pub fn is_valid_track(s: &str) -> bool {
    ALL_TRACKS.contains(&s)
}

// ── Magic token tables ──────────────────────────────────────────

/// Glyphs that are NOT specific to a single track and need resolution.
const STATIC_MAGIC_TOKENS: &[&str] = &[
    "s", "S", "b", "B", "b2", "B2", "r", "R", "r2", "R2",
    "c", "C", "c2", "C2", "t1", "T1", "t2", "T2", "t3", "T3", "t4", "T4",
    "o", "O", "spl", "SPL", "chn", "CHN", "cb", "CB", "wb", "WB", "cl", "CL",
];

/// Uppercase magic tokens that receive an automatic "accent" modifier
/// (unless on ST sticking track).
const ACCENT_MAGIC_TOKENS: &[&str] = &[
    "D", "X", "P", "G", "S", "B", "B2",
    "R", "R2", "C", "C2", "O", "SPL",
    "CHN", "CB", "WB", "CL",
];

fn has_modifier(modifiers: &[String], name: &str) -> bool {
    modifiers.iter().any(|m| m == name)
}

// ── Resolution ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ResolvedToken {
    pub track: String,
    pub glyph: String,
    pub modifiers: Vec<String>,
}

/// Resolve a magic token glyph to a (track, glyph, modifiers) tuple.
/// Returns None for rest tokens (`-`).
pub fn resolve_token(
    glyph: &str,
    context_track: Option<&str>,
    track_override: Option<&str>,
    incoming_modifiers: &[String],
) -> Option<ResolvedToken> {
    if glyph == "-" { return None; }

    let mut modifiers: Vec<String> = incoming_modifiers.to_vec();
    let is_sticking = glyph == "R" || glyph == "L";

    // 1. Resolve track
    let track = if let Some(t) = track_override {
        t.to_string()
    } else if context_track == Some("ST") && is_sticking {
        "ST".to_string()
    } else if STATIC_MAGIC_TOKENS.contains(&glyph) {
        resolve_fallback_track(glyph)
    } else if let Some(ct) = context_track {
        ct.to_string()
    } else {
        resolve_fallback_track(glyph)
    };

    // 2. Accent magic tokens
    if ACCENT_MAGIC_TOKENS.contains(&glyph)
        && !(track == "ST" && is_sticking)
        && !has_modifier(&modifiers, "accent")
    {
        modifiers.push("accent".to_string());
    }

    // 3. Ghost/open modifiers
    if (glyph == "g" || glyph == "G") && !has_modifier(&modifiers, "ghost") {
        modifiers.push("ghost".to_string());
    }
    if (glyph == "o" || glyph == "O") && !has_modifier(&modifiers, "open") {
        modifiers.push("open".to_string());
    }

    // 4. Context-aware x/X → cross for drum family
    if (glyph == "x" || glyph == "X")
        && get_track_family(&track) == TrackFamily::Drum
        && !has_modifier(&modifiers, "cross")
    {
        modifiers.push("cross".to_string());
    }

    // 5. Notehead selection
    let glyph = if track == "ST" {
        glyph.to_string()
    } else if get_track_family(&track) == TrackFamily::Cymbal {
        "x".to_string()
    } else {
        "d".to_string()
    };

    Some(ResolvedToken { track, glyph, modifiers })
}

/// Map a magic token glyph to its default track.
pub fn resolve_fallback_track(glyph: &str) -> String {
    match glyph {
        "s" | "S" => "SD",
        "b" | "B" => "BD",
        "b2" | "B2" => "BD2",
        "t1" | "T1" => "T1",
        "t2" | "T2" => "T2",
        "t3" | "T3" => "T3",
        "t4" | "T4" => "T4",
        "c" | "C" => "C",
        "c2" | "C2" => "C2",
        "r" | "R" => "RC",
        "r2" | "R2" => "RC2",
        "spl" | "SPL" => "SPL",
        "chn" | "CHN" => "CHN",
        "cb" | "CB" => "CB",
        "wb" | "WB" => "WB",
        "cl" | "CL" => "CL",
        "p" | "P" => "HF",
        "g" | "G" => "SD",
        _ => "HH",
    }.to_string()
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_assignment() {
        assert_eq!(voice_for_track("BD"), 2);
        assert_eq!(voice_for_track("BD2"), 2);
        assert_eq!(voice_for_track("HF"), 2);
        assert_eq!(voice_for_track("SD"), 1);
        assert_eq!(voice_for_track("HH"), 1);
        assert_eq!(voice_for_track("T1"), 1);
    }

    #[test]
    fn test_fallback_track() {
        assert_eq!(resolve_fallback_track("s"), "SD");
        assert_eq!(resolve_fallback_track("b"), "BD");
        assert_eq!(resolve_fallback_track("c"), "C");
        assert_eq!(resolve_fallback_track("x"), "HH");
        assert_eq!(resolve_fallback_track("d"), "HH");
    }

    #[test]
    fn test_resolve_magic_accent() {
        // `D` → not in STATIC_MAGIC_TOKENS, fallback → "HH"
        // (D is only in ACCENT_MAGIC_TOKENS, not STATIC)
        let r = resolve_token("D", None, None, &[]).unwrap();
        assert_eq!(r.track, "HH");
        assert!(has_modifier(&r.modifiers, "accent"));
    }

    #[test]
    fn test_resolve_sticking() {
        let r = resolve_token("R", Some("ST"), None, &[]).unwrap();
        assert_eq!(r.track, "ST");
        assert!(!has_modifier(&r.modifiers, "accent"));
    }

    #[test]
    fn test_resolve_drum_cross() {
        let r = resolve_token("x", Some("SD"), None, &[]).unwrap();
        assert_eq!(r.track, "SD");
        assert_eq!(r.glyph, "d");
        assert!(has_modifier(&r.modifiers, "cross"));
    }

    #[test]
    fn test_track_override() {
        let r = resolve_token("x", Some("HH"), Some("SD"), &[]).unwrap();
        assert_eq!(r.track, "SD");
    }
}
