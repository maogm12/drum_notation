use crate::fraction::{Fraction, calculate_token_weight_as_fraction, fractions_equal};
use crate::resolve::{resolve_token, voice_for_track};
use crate::hairpin::HairpinKind;

// ── Normalized Event ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NormalizedEvent {
    pub track: String,
    pub paragraph_index: u32,
    pub measure_index: u32,
    pub measure_in_paragraph: u32,
    pub start: Fraction,
    pub duration: Fraction,
    pub kind: EventKind,
    pub glyph: String,
    pub modifiers: Vec<String>,
    pub modifier: Option<String>,
    pub voice: u8,
    pub beam: String,
    pub tuplet: Option<(u32, u32)>, // (actual, normal)
    pub source_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Hit,
    Rest,
    Sticking,
}

// ── Internal Token Types (mirrors MeasureExpr from parser) ───────

#[derive(Debug, Clone)]
pub enum TokenGlyph {
    Basic {
        value: String,
        dots: u32,
        halves: u32,
        stars: u32,
        modifiers: Vec<String>,
        track_override: Option<String>,
    },
    Combined {
        items: Vec<TokenGlyph>,
    },
    Group {
        count: u32,
        span: u32,
        items: Vec<TokenGlyph>,
        modifiers: Vec<String>,
    },
    Braced {
        track: String,
        items: Vec<TokenGlyph>,
    },
    Crescendo,
    Decrescendo,
    HairpinEnd,
}

// ── Token → Event Expansion ─────────────────────────────────────

pub fn token_to_events(
    token: &TokenGlyph,
    start: Fraction,
    duration: Fraction,
    context_track: Option<&str>,
    paragraph_index: u32,
    measure_index: u32,
    measure_in_paragraph: u32,
    inherited_tuplet: Option<(u32, u32)>,
    source_offset: u32,
) -> Vec<NormalizedEvent> {
    match token {
        TokenGlyph::Basic { value, dots: _, halves: _, stars: _, modifiers, track_override } => {
            if value == "-" { return vec![]; }

            let resolved = resolve_token(value, context_track, track_override.as_deref(), modifiers);
            let resolved = match resolved {
                Some(r) => r,
                None => return vec![],
            };

            let primary_modifier = resolved.modifiers.iter()
                .find(|m| *m != "accent")
                .cloned();

            let kind = if resolved.track == "ST" { EventKind::Sticking } else { EventKind::Hit };

            let event = NormalizedEvent {
                track: resolved.track.clone(),
                paragraph_index,
                measure_index,
                measure_in_paragraph,
                start,
                duration,
                kind,
                glyph: resolved.glyph.clone(),
                modifiers: resolved.modifiers.clone(),
                modifier: primary_modifier,
                voice: voice_for_track(&resolved.track),
                beam: "none".to_string(),
                tuplet: inherited_tuplet,
                source_offset,
            };
            vec![event]
        }
        TokenGlyph::Combined { items } => {
            items.iter()
                .flat_map(|item| token_to_events(item, start, duration, context_track, paragraph_index, measure_index, measure_in_paragraph, inherited_tuplet, source_offset))
                .collect()
        }
        TokenGlyph::Braced { track, items } => {
            let mut events = Vec::new();
            let total_weight = braced_total_weight(items);

            if fractions_equal(total_weight, Fraction::zero()) {
                return events;
            }

            let mut current_start = start;
            for item in items {
                let item_weight = item_weight(item);
                let item_duration = duration
                    .multiply(item_weight)
                    .divide(total_weight);

                events.extend(token_to_events(
                    item,
                    current_start,
                    item_duration,
                    Some(track),
                    paragraph_index,
                    measure_index,
                    measure_in_paragraph,
                    inherited_tuplet,
                    source_offset,
                ));
                current_start = current_start.add(item_duration);
            }
            events
        }
        TokenGlyph::Group { count, span, items, modifiers } => {
            let mut events = Vec::new();
            let total_weight = group_total_weight(items);

            if fractions_equal(total_weight, Fraction::zero()) {
                return events;
            }

            let count = *count;
            let span = *span;
            let effective_count = if count == 0 { items.len() as u32 } else { count };
            let effective_span = span;
            // Only mark as tuplet if there's actual compression/expansion
            let group_tuplet = if effective_count != span && effective_count > effective_span
            {
                Some((effective_count, effective_span))
            } else {
                inherited_tuplet
            };

            let mut current_start = start;
            for item in items {
                let mut item = item.clone();
                // Apply group modifiers
                if !modifiers.is_empty() {
                    apply_modifiers(&mut item, modifiers);
                }
                let item_weight = item_weight(&item);
                let item_duration = duration
                    .multiply(item_weight)
                    .divide(total_weight);

                events.extend(token_to_events(
                    &item,
                    current_start,
                    item_duration,
                    context_track,
                    paragraph_index,
                    measure_index,
                    measure_in_paragraph,
                    group_tuplet,
                    source_offset,
                ));
                current_start = current_start.add(item_duration);
            }
            events
        }
        TokenGlyph::Crescendo | TokenGlyph::Decrescendo | TokenGlyph::HairpinEnd => {
            vec![]
        }
    }
}

// ── Weight Helpers ───────────────────────────────────────────────

fn item_weight(token: &TokenGlyph) -> Fraction {
    match token {
        TokenGlyph::Basic { dots, halves, stars, .. } => {
            calculate_token_weight_as_fraction(*dots, *stars, *halves, None)
        }
        TokenGlyph::Group { span, .. } => {
            // Group weight = span (normal duration)
            Fraction::new(*span as u64, 1)
        }
        TokenGlyph::Combined { items } => {
            // max weight of items
            items.iter()
                .map(item_weight)
                .max_by(|a, b| a.compare(*b))
                .unwrap_or(Fraction::zero())
        }
        TokenGlyph::Braced { items, .. } => braced_total_weight(items),
        TokenGlyph::Crescendo | TokenGlyph::Decrescendo | TokenGlyph::HairpinEnd => {
            Fraction::zero()
        }
    }
}

fn group_total_weight(items: &[TokenGlyph]) -> Fraction {
    items.iter()
        .map(item_weight)
        .fold(Fraction::zero(), |a, b| a.add(b))
}

fn braced_total_weight(items: &[TokenGlyph]) -> Fraction {
    items.iter()
        .map(item_weight)
        .fold(Fraction::zero(), |a, b| a.add(b))
}

fn apply_modifiers(token: &mut TokenGlyph, mods: &[String]) {
    if let TokenGlyph::Basic { modifiers, .. } = token {
        for m in mods {
            if !modifiers.contains(m) {
                modifiers.push(m.clone());
            }
        }
    }
}

// ── Hairpin scanning ─────────────────────────────────────────────

/// Scan a measure's tokens for hairpin events.
/// Returns list of (start, open_kind_or_none, close_or_none).
pub fn scan_hairpin_tokens(
    tokens: &[TokenGlyph],
    _measure_start: Fraction,
    _divisions: u32,
) -> Vec<(Fraction, Option<HairpinKind>, Option<()>)> {
    let mut results = Vec::new();
    let mut position = Fraction::zero();

    for token in tokens {
        match token {
            TokenGlyph::Crescendo => {
                results.push((position, Some(HairpinKind::Crescendo), None));
            }
            TokenGlyph::Decrescendo => {
                results.push((position, Some(HairpinKind::Decrescendo), None));
            }
            TokenGlyph::HairpinEnd => {
                results.push((position, None, Some(())));
            }
            _ => {
                let w = item_weight(token);
                position = position.add(w);
            }
        }
    }

    results
}
