use crate::fraction::{Fraction, calculate_token_weight_as_fraction};
use crate::resolve::{get_track_family, TrackFamily};
use crate::validate::{validate_modifier_legality, validate_grouping};
use crate::hairpin::{HairpinState, HairpinIntent, collect_track_hairpins, close_dangling_hairpin};
use crate::nav::{StartNav, EndNav, Anchor, BarlineType};
use crate::volta::{VoltaMeasure, propagate_voltas};
use crate::event::{NormalizedEvent, TokenGlyph, token_to_events, scan_hairpin_tokens};
use crate::ast::{Document, Barline, MeasureExpr, TrackLine, HeaderSection, SourceLocation};
use std::collections::{HashMap, HashSet};

// ── Inline-Repeat Expansion Helpers ────────────────────────────────

/// Expanded form of a MeasureSection after inline-repeat processing.
#[derive(Debug, Clone)]
struct ExpandedSection {
    tokens: Vec<MeasureExpr>,
    barline: Barline,
    barline_location: SourceLocation,
    closing_barline: Option<Barline>,
    closing_barline_location: Option<SourceLocation>,
}

/// Split tokens into content and optional trailing InlineRepeat(n).
fn split_inline_repeat(tokens: &[MeasureExpr]) -> (Vec<MeasureExpr>, Option<i32>) {
    let mut content = Vec::new();
    let mut inline_repeat = None;
    for tok in tokens {
        match tok {
            MeasureExpr::InlineRepeat(n) => inline_repeat = Some(*n),
            _ => content.push(tok.clone()),
        }
    }
    (content, inline_repeat)
}

/// Expand a TrackLine's measure sections, resolving inline repeats.
fn expand_line_sections(line: &TrackLine, errors: &mut Vec<String>) -> Vec<ExpandedSection> {
    let mut result = Vec::new();
    let mut _prev_tokens: Option<Vec<MeasureExpr>> = None;

    for section in &line.measures {
        let (content, repeat) = split_inline_repeat(&section.tokens);

        if let Some(n) = repeat {
            if n <= 0 {
                errors.push("Repeat count must be at least 1".to_string());
                result.push(ExpandedSection {
                    tokens: content.clone(),
                    barline: section.barline.clone(),
                    barline_location: section.barline_location.clone(),
                    closing_barline: section.closing_barline.clone(),
                    closing_barline_location: section.closing_barline_location.clone(),
                });
            } else {
                // Expand content into n total copies
                for i in 0..n as usize {
                    let is_first = i == 0;
                    let is_last = i + 1 == n as usize;
                    result.push(ExpandedSection {
                        tokens: content.clone(),
                        barline: if is_first { section.barline.clone() } else { Barline::Regular },
                        barline_location: section.barline_location.clone(),
                        closing_barline: if is_last { section.closing_barline.clone() } else { None },
                        closing_barline_location: if is_last { section.closing_barline_location.clone() } else { None },
                    });
                }
            }
            if !content.is_empty() {
                _prev_tokens = Some(content);
            }
        } else {
            // No inline repeat: use section as-is
            result.push(ExpandedSection {
                tokens: content,
                barline: section.barline.clone(),
                barline_location: section.barline_location.clone(),
                closing_barline: section.closing_barline.clone(),
                closing_barline_location: section.closing_barline_location.clone(),
            });
            if !section.tokens.is_empty() {
                _prev_tokens = Some(section.tokens.clone());
            }
        }
    }
    result
}

// ── Normalized Score Output Types ────────────────────────────────

#[derive(Debug, Clone)]
pub struct NormalizedScore {
    pub version: String,
    pub header: NormalizedHeader,
    pub tracks: Vec<NormalizedTrack>,
    pub measures: Vec<NormalizedMeasure>,
    pub errors: Vec<String>,
    pub repeat_spans: Vec<RepeatSpan>,
}

#[derive(Debug, Clone)]
pub struct NormalizedHeader {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub composer: Option<String>,
    pub tempo: u32,
    pub time_beats: u32,
    pub time_beat_unit: u32,
    pub divisions: u32,
    pub note_value: u32,
    pub grouping: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct NormalizedTrack {
    pub id: String,
    pub family: String,
}

#[derive(Debug, Clone)]
pub struct NormalizedMeasure {
    pub index: u32,
    pub global_index: u32,
    pub paragraph_index: u32,
    pub measure_in_paragraph: u32,
    pub source_line: u32,
    pub events: Vec<NormalizedEvent>,
    pub barline: Option<String>,
    pub closing_barline: Option<String>,
    pub start_nav: Option<StartNav>,
    pub end_nav: Option<EndNav>,
    pub volta: Option<Vec<u32>>,
    pub hairpins: Vec<HairpinIntent>,
    pub measure_repeat_slashes: Option<u32>,
    pub multi_rest_count: Option<u32>,
    pub note_value: u32,
    pub volta_terminator: bool,
}

#[derive(Debug, Clone)]
pub struct RepeatSpan {
    pub start_measure: u32,
    pub end_measure: u32,
    pub times: u32,
}

// ── Conversion Helpers ───────────────────────────────────────────

fn to_token_glyph(expr: &MeasureExpr) -> TokenGlyph {
    match expr {
        MeasureExpr::BasicNote(n) => TokenGlyph::Basic {
            value: n.glyph.clone(),
            dots: n.dots,
            halves: n.halves,
            stars: n.stars,
            modifiers: n.modifiers.clone(),
            track_override: None,
        },
        MeasureExpr::SummonedNote { track, note } => TokenGlyph::Basic {
            value: note.glyph.clone(),
            dots: note.dots,
            halves: note.halves,
            stars: note.stars,
            modifiers: note.modifiers.clone(),
            track_override: Some(track.clone()),
        },
        MeasureExpr::CombinedHit(hits) => TokenGlyph::Combined {
            items: hits.iter().map(|e| match e {
                MeasureExpr::BasicNote(n) => TokenGlyph::Basic {
                    value: n.glyph.clone(),
                    dots: n.dots,
                    halves: n.halves,
                    stars: n.stars,
                    modifiers: n.modifiers.clone(),
                    track_override: None,
                },
                MeasureExpr::SummonedNote { track, note } => TokenGlyph::Basic {
                    value: note.glyph.clone(),
                    dots: note.dots,
                    halves: note.halves,
                    stars: note.stars,
                    modifiers: note.modifiers.clone(),
                    track_override: Some(track.clone()),
                },
                _ => TokenGlyph::Basic {
                    value: "-".to_string(), dots: 0, halves: 0, stars: 0,
                    modifiers: vec![], track_override: None,
                },
            }).collect(),
        },
        MeasureExpr::Group(g) => {
            // Check if any item is itself a Group (nested group)
            let has_nested = g.items.iter().any(|i| matches!(i, MeasureExpr::Group(_)));
            TokenGlyph::Group {
                // In drummark [N: ...] notation, N is the SPAN (normal duration),
                // not the tuplet numerator. Items count is the actual note count.
                // Without N (no ratio): span defaults to 1.
                // For nested groups, match Lezer: treat outer as if span=items.len()
                count: g.items.len() as u32,
                span: if has_nested { g.items.len() as u32 } else { g.n.unwrap_or(1) },
                items: g.items.iter().map(to_token_glyph).collect(),
                modifiers: g.modifiers.clone(),
            }
        }
        MeasureExpr::RoutedBracedBlock { track, content } => TokenGlyph::Braced {
            track: track.clone(),
            items: content.iter().map(to_token_glyph).collect(),
        },
        MeasureExpr::InlineBracedBlock(content) => {
            // Inline braced blocks don't have a track override; use anonymous
            TokenGlyph::Braced {
                track: "SD".to_string(), // fallback
                items: content.iter().map(to_token_glyph).collect(),
            }
        }
        MeasureExpr::Crescendo => TokenGlyph::Crescendo,
        MeasureExpr::Decrescendo => TokenGlyph::Decrescendo,
        MeasureExpr::HairpinEnd => TokenGlyph::HairpinEnd,
        MeasureExpr::MeasureRepeat(_count) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
        MeasureExpr::MultiRest(_count) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
        MeasureExpr::InlineRepeat(_) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
        MeasureExpr::NavMarker(_) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
        MeasureExpr::NavJump(_) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
    }
}

fn barline_type(bl: &Barline) -> Option<String> {
    match bl {
        Barline::Regular => Some("regular".to_string()),
        Barline::Double => Some("double".to_string()),
        Barline::RepeatStart => Some("repeat-start".to_string()),
        Barline::RepeatEnd => Some("repeat-end".to_string()),
        Barline::VoltaTerminator => Some("regular".to_string()),
        Barline::RepeatEndVoltaTerminator | Barline::DoubleVoltaTerminator => None,
        Barline::VoltaRepeatStart => Some("repeat-start".to_string()),
        Barline::Volta { prefix, numbers: _numbers } => {
            if prefix == "|:" { Some("repeat-start".to_string()) }
            else if prefix == ":|" { Some("repeat-end".to_string()) }
            else { Some("regular".to_string()) }
        }
    }
}

fn default_note_value(headers: &HeaderSection) -> u32 {
    headers.note.map(|(_, d)| d).unwrap_or(8)
}

// ── Main Normalizer ─────────────────────────────────────────────

pub fn normalize_document(doc: &Document) -> NormalizedScore {
    let mut errors: Vec<String> = Vec::new();

    // Extract header
    let hs = &doc.headers;
    let time_beats = hs.time.unwrap_or((4, 4)).0;
    let time_beat_unit = hs.time.unwrap_or((4, 4)).1;
    let divisions = hs.divisions.unwrap_or(16);
    let note_value = default_note_value(hs);
    let grouping = hs.grouping.clone().unwrap_or_else(|| vec![1]);

    // Validate grouping
    if let Some(err) = validate_grouping(&grouping, time_beats, time_beat_unit, divisions) {
        errors.push(err);
    }

    let header = NormalizedHeader {
        title: hs.title.clone(),
        subtitle: hs.subtitle.clone(),
        composer: hs.composer.clone(),
        tempo: hs.tempo.unwrap_or(120),
        time_beats,
        time_beat_unit,
        divisions,
        note_value,
        grouping: grouping.clone(),
    };

    // Collect all tracks
    let mut track_set: HashSet<String> = HashSet::new();
    for para in &doc.paragraphs {
        for line in &para.lines {
            if let Some(ref t) = line.track {
                track_set.insert(t.clone());
            }
        }
    }

    let mut tracks: Vec<NormalizedTrack> = track_set.iter().map(|id| {
        let family = match get_track_family(id) {
            TrackFamily::Cymbal => "cymbal",
            TrackFamily::Drum => "drum",
            TrackFamily::Pedal => "pedal",
            TrackFamily::Percussion => "percussion",
            TrackFamily::Auxiliary => "auxiliary",
        };
        NormalizedTrack { id: id.clone(), family: family.to_string() }
    }).collect();
    tracks.sort_by(|a, b| a.id.cmp(&b.id));

    // ── Main Pass: walk paragraphs → measures → tracks → tokens ──

    let mut all_measures: Vec<NormalizedMeasure> = Vec::new();
    let mut global_index: u32 = 0;

    // Per-track hairpin state
    let mut hairpin_states: HashMap<String, HairpinState> = HashMap::new();

    for (para_idx, para) in doc.paragraphs.iter().enumerate() {
        let para_note_value = para.note.map(|(_, d)| d).unwrap_or(note_value);

        // ── Inline-repeat expansion ──
        let expanded_lines: Vec<Vec<ExpandedSection>> = para.lines.iter()
            .map(|line| expand_line_sections(line, &mut errors))
            .collect();
        let measure_count = expanded_lines.iter()
            .map(|l| l.len())
            .max()
            .unwrap_or(0);

        for m_idx in 0..measure_count {
            let mut measure_events: Vec<NormalizedEvent> = Vec::new();
            let mut measure_hairpins: Vec<HairpinIntent> = Vec::new();
            let mut barline: Option<String> = None;
            let mut closing_barline: Option<String> = None;
            let mut repeat_start = false;
            let mut repeat_end = false;
            let mut volta_indices: Option<Vec<u32>> = None;
            let mut volta_terminator = false;
            let mut measure_repeat_slashes: Option<u32> = None;
            let mut multi_rest_count: Option<u32> = None;
            let mut start_nav: Option<StartNav> = None;
            let mut end_nav: Option<EndNav> = None;

            for (li, line) in para.lines.iter().enumerate() {
                let expanded = &expanded_lines[li];
                // Pad shorter tracks by repeating the last section
                let (es, is_padded) = if m_idx < expanded.len() {
                    (&expanded[m_idx], false)
                } else if let Some(last) = expanded.last() {
                    (last, true)
                } else {
                    continue;
                };
                let context_track = line.track.as_deref();
                let use_track = context_track.unwrap_or("ANONYMOUS");

                // Barline metadata from first line
                if barline.is_none() {
                    barline = barline_type(&es.barline);
                }

                // Check for repeat/volta metadata
                match &es.barline {
                    Barline::RepeatStart | Barline::VoltaRepeatStart => repeat_start = true,
                    Barline::RepeatEnd => repeat_end = true,
                    Barline::Volta { numbers, .. } => {
                        volta_indices = Some(numbers.clone());
                    }
                    Barline::VoltaTerminator | Barline::RepeatEndVoltaTerminator | Barline::DoubleVoltaTerminator => {
                        volta_terminator = true;
                    }
                    _ => {}
                }

                // Check closing barline for repeat-end and volta terminator
                if let Some(ref cb) = es.closing_barline {
                    match cb {
                        Barline::RepeatEnd => {
                            repeat_end = true;
                            closing_barline = Some("repeat-end".to_string());
                        }
                        Barline::Double => {
                            closing_barline = Some("double".to_string());
                            if barline.is_none() || barline.as_deref() == Some("regular") {
                                barline = Some("double".to_string());
                            }
                        }
                        Barline::VoltaTerminator => {
                            volta_terminator = true;
                        }
                        Barline::RepeatEndVoltaTerminator => {
                            repeat_end = true;
                            volta_terminator = true;
                            closing_barline = Some("repeat-end".to_string());
                        }
                        Barline::DoubleVoltaTerminator => {
                            volta_terminator = true;
                            closing_barline = Some("double".to_string());
                            if barline.is_none() || barline.as_deref() == Some("regular") {
                                barline = Some("double".to_string());
                            }
                        }
                        _ => {}
                    }
                }

                // Scan tokens — filter out zero-time markers before converting
                let tokens: Vec<TokenGlyph> = es.tokens.iter()
                    .filter(|t| !matches!(t,
                        MeasureExpr::NavMarker(_) | MeasureExpr::NavJump(_)
                        | MeasureExpr::MeasureRepeat(_) | MeasureExpr::MultiRest(_)
                    ))
                    .map(to_token_glyph)
                    .collect();

                // Scan for measure-repeat, multi-rest, nav markers
                // Skip metadata on padded sections to avoid repeating non-note data
                if !is_padded {
                    for tok in &es.tokens {
                    match tok {
                        MeasureExpr::MeasureRepeat(count) => {
                            measure_repeat_slashes = Some(*count);
                        }
                        MeasureExpr::MultiRest(count) => {
                            multi_rest_count = Some(*count);
                        }
                        MeasureExpr::NavMarker(name) => {
                            start_nav = Some(match name.as_str() {
                                "segno" => StartNav::Segno { anchor: Anchor::LeftEdge },
                                "coda" => StartNav::Coda { anchor: Anchor::LeftEdge },
                                _ => continue,
                            });
                        }
                        MeasureExpr::NavJump(name) => {
                            end_nav = Some(match name.as_str() {
                                "fine" => EndNav::Fine { anchor: Anchor::RightEdge },
                                "dc" => EndNav::DC { anchor: Anchor::RightEdge },
                                "ds" => EndNav::DS { anchor: Anchor::RightEdge },
                                "dc-al-fine" => EndNav::DCalFine { anchor: Anchor::RightEdge },
                                "dc-al-coda" => EndNav::DCalCoda { anchor: Anchor::RightEdge },
                                "ds-al-fine" => EndNav::DSalFine { anchor: Anchor::RightEdge },
                                "ds-al-coda" => EndNav::DSalCoda { anchor: Anchor::RightEdge },
                                "to-coda" => EndNav::ToCoda { anchor: Anchor::RightEdge },
                                _ => continue,
                            });
                        }
                        _ => {}
                    }
                }
                } // !is_padded

                // Convert tokens to events
                let duration_per_quarter = Fraction::new(1, para_note_value as u64);
                let _duration: Fraction = Fraction::zero();

                // Calculate total weight for measure duration
                let mut total_weight = Fraction::zero();
                for t in &tokens {
                    total_weight = total_weight.add(token_weight(t));
                }
                let _measure_duration = total_weight.multiply(duration_per_quarter);

                // Expand tokens to events
                let mut position = Fraction::zero();
                for token in tokens.iter() {
                    let token_start = position.multiply(duration_per_quarter);
                    // Validate modifier legality
                    if let TokenGlyph::Basic { value, modifiers, .. } = token {
                        if value != "-" {
                            if let Some(resolved) = crate::resolve::resolve_token(
                                value,
                                context_track,
                                None,
                                modifiers,
                            ) {
                                for m in &resolved.modifiers {
                                    if let Some(err) = validate_modifier_legality(m, &resolved.track) {
                                        errors.push(err);
                                    }
                                }
                            }
                        }
                    }

                    let weight = token_weight(token);
                    let dur = weight.multiply(duration_per_quarter);

                    let events = token_to_events(
                        token,
                        token_start,
                        dur,
                        context_track,
                        para_idx as u32,
                        global_index,
                        m_idx as u32,
                        None,
                        0, // source_offset
                    );
                    measure_events.extend(events);
                    position = position.add(weight);
                }

                // Collect hairpins per track (positions are raw weight counts — convert to time)
                let hairpin_scan = scan_hairpin_tokens(&tokens, Fraction::zero(), divisions);
                let hairpin_scan_time: Vec<_> = hairpin_scan.iter().map(|(pos, kind, sig)| {
                    (pos.multiply(duration_per_quarter), *kind, *sig)
                }).collect();
                let state = hairpin_states.entry(use_track.to_string())
                    .or_default();
                let track_hairpins = collect_track_hairpins(&hairpin_scan_time, global_index as usize, state);
                measure_hairpins.extend(track_hairpins);
            }

            // End-nav barline forcing
            if let Some(ref en) = end_nav {
                match en.forced_barline() {
                    BarlineType::Final => barline = Some("final".to_string()),
                    BarlineType::Double => barline = Some("double".to_string()),
                    _ => {}
                }
            }

            // Post-process barline: merge repeat-start + repeat-end into repeat-both
            if repeat_start && repeat_end {
                barline = Some("repeat-both".to_string());
            } else if repeat_start {
                barline = barline.or(Some("repeat-start".to_string()));
            } else if repeat_end {
                barline = Some("repeat-end".to_string());
            }

            all_measures.push(NormalizedMeasure {
                index: m_idx as u32,
                global_index,
                paragraph_index: para_idx as u32,
                measure_in_paragraph: m_idx as u32,
                source_line: 0,
                events: measure_events,
                barline,
                closing_barline,
                start_nav,
                end_nav,
                volta: volta_indices,
                hairpins: measure_hairpins,
                measure_repeat_slashes,
                multi_rest_count,
                note_value: para_note_value,
                volta_terminator,
            });

            global_index += 1;
        }
    }

    // ── Post-pass 1: Volta propagation ───────────────────────────

    let mut volta_measures: Vec<VoltaMeasure> = all_measures.iter().map(|m| VoltaMeasure {
        seed_volta: m.volta.clone(),
        volta: None,
        repeat_end: m.barline.as_deref() == Some("repeat-end"),
        repeat_both: m.barline.as_deref() == Some("repeat-both"),
        volta_terminator: m.volta_terminator,
        paragraph_index: m.paragraph_index,
    }).collect();

    propagate_voltas(&mut volta_measures);

    for (i, vm) in volta_measures.iter().enumerate() {
        all_measures[i].volta = vm.volta.as_ref().map(|v| v.indices.clone());
    }

    // ── Post-pass 2: Derive repeat spans ─────────────────────────

    let repeat_spans = derive_repeat_spans(&all_measures, &mut errors);

    // ── Post-pass 3: Close dangling hairpins ─────────────────────

    let last_idx = all_measures.len().saturating_sub(1);
    for (_track_id, state) in hairpin_states.iter_mut() {
        if let Some(hairpin) = close_dangling_hairpin(state, last_idx, Fraction::new(1, 1)) {
            if let Some(m) = all_measures.last_mut() {
                m.hairpins.push(hairpin);
            }
        }
    }

    // ── Post-pass 4: Final barline ───────────────────────────────

    if let Some(last) = all_measures.last_mut() {
        if last.barline.is_none() || last.barline.as_deref() == Some("regular") {
            last.barline = Some("final".to_string());
        }
    }

    NormalizedScore {
        version: "1.0".to_string(),
        header,
        tracks,
        measures: all_measures,
        errors,
        repeat_spans,
    }
}

fn derive_repeat_spans(measures: &[NormalizedMeasure], errors: &mut Vec<String>) -> Vec<RepeatSpan> {
    let mut repeat_spans = Vec::new();
    let mut open_start: Option<u32> = None;
    let mut open_start_has_ending = false;

    for (index, measure) in measures.iter().enumerate() {
        let start = matches!(measure.barline.as_deref(), Some("repeat-start") | Some("repeat-both"));
        let end = matches!(measure.barline.as_deref(), Some("repeat-end") | Some("repeat-both"))
            || matches!(measure.closing_barline.as_deref(), Some("repeat-end"));
        let next_volta = measures.get(index + 1).and_then(|next| next.volta.clone());

        if start && !end && open_start.is_some() {
            errors.push(format!(
                "Line {}: Nested repeat start at bar {} is not supported",
                measure.source_line.max(1),
                measure.global_index + 1
            ));
        }
        if end && open_start.is_none() && !start {
            errors.push(format!(
                "Line {}: Repeat end at bar {} has no matching start",
                measure.source_line.max(1),
                measure.global_index + 1
            ));
        }

        if end {
            if let Some(start_measure) = open_start {
                repeat_spans.push(RepeatSpan {
                    start_measure,
                    end_measure: measure.global_index,
                    times: 2,
                });
                open_start_has_ending = true;
                if next_volta.is_none() {
                    open_start = None;
                    open_start_has_ending = false;
                }
            }
        } else if open_start.is_some() && open_start_has_ending && next_volta.is_none() {
            open_start = None;
            open_start_has_ending = false;
        }

        if start && (open_start.is_none() || end) {
            open_start = Some(measure.global_index);
            open_start_has_ending = false;
        }
    }

    if let Some(start_measure) = open_start {
        if !open_start_has_ending {
            errors.push(format!(
                "Line 1: Repeat starting at bar {} is missing an end",
                start_measure + 1
            ));
        }
    }

    repeat_spans
}

fn token_weight(token: &TokenGlyph) -> Fraction {
    match token {
        TokenGlyph::Basic { dots, halves, stars, .. } => {
            calculate_token_weight_as_fraction(*dots, *stars, *halves, None)
        }
        TokenGlyph::Group { span, .. } => {
            // Group total weight = span (normal duration the group occupies)
            Fraction::new(*span as u64, 1)
        }
        TokenGlyph::Combined { items } => {
            items.iter().map(token_weight)
                .max_by(|a, b| a.compare(*b))
                .unwrap_or(Fraction::zero())
        }
        TokenGlyph::Braced { items, .. } => {
            items.iter().map(token_weight)
                .fold(Fraction::zero(), |a, b| a.add(b))
        }
        TokenGlyph::Crescendo | TokenGlyph::Decrescendo | TokenGlyph::HairpinEnd => {
            Fraction::zero()
        }
    }
}

#[cfg(test)]
mod volta_test {
    use crate::parser;
    use crate::normalize;

    #[test]
    fn test_volta_barlines() {
        let p = parser::Parser::new("|: s s |1. d d :|2. g g |");
        let doc = p.parse().unwrap();
        let score = normalize::normalize_document(&doc);
        assert_eq!(score.measures.len(), 3);
        assert_eq!(score.measures[1].volta, Some(vec![1]));
        assert_eq!(score.measures[2].volta, Some(vec![2]));
        assert_eq!(score.repeat_spans.len(), 1);
        assert_eq!(score.repeat_spans[0].start_measure, 0);
        assert_eq!(score.repeat_spans[0].end_measure, 1);
    }

    #[test]
    fn test_volta_continues_across_paragraph_after_repeat_end() {
        let p = parser::Parser::new(
            "time 4/4\nnote 1/8\ngrouping 2+2\n\n|: ssss ssss |1. ssSs ssSs :|2. cCcc cccc :|\n\n| ssss ssss |3. bbbb bbbb |.\n",
        );
        let doc = p.parse().unwrap();
        let score = normalize::normalize_document(&doc);
        assert_eq!(score.errors, Vec::<String>::new());
        assert_eq!(score.measures.len(), 5);
        assert_eq!(score.measures[0].barline.as_deref(), Some("repeat-start"));
        assert_eq!(score.measures[0].volta, None);
        assert_eq!(score.measures[1].barline.as_deref(), Some("repeat-end"));
        assert_eq!(score.measures[1].volta, Some(vec![1]));
        assert_eq!(score.measures[2].barline.as_deref(), Some("repeat-end"));
        assert_eq!(score.measures[2].volta, Some(vec![2]));
        assert_eq!(score.measures[3].barline.as_deref(), Some("regular"));
        assert_eq!(score.measures[3].volta, Some(vec![2]));
        assert_eq!(score.measures[4].barline.as_deref(), Some("final"));
        assert_eq!(score.measures[4].volta, Some(vec![3]));
    }

    #[test]
    fn test_closing_double_barline_survives_normalization() {
        let p = parser::Parser::new("time 4/4\nnote 1/8\ngrouping 2+2\nSD | d ||\n");
        let doc = p.parse().unwrap();
        let score = normalize::normalize_document(&doc);
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].barline.as_deref(), Some("double"));
        assert_eq!(score.measures[0].closing_barline.as_deref(), Some("double"));
    }

    #[test]
    fn test_opening_repeat_and_closing_double_are_both_preserved() {
        let p = parser::Parser::new("time 4/4\nnote 1/8\ngrouping 2+2\nSD |: d ||\n");
        let doc = p.parse().unwrap();
        let score = normalize::normalize_document(&doc);
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].barline.as_deref(), Some("repeat-start"));
        assert_eq!(score.measures[0].closing_barline.as_deref(), Some("double"));
    }

    #[test]
    fn test_double_volta_terminator_preserves_double_not_repeat_end() {
        let p = parser::Parser::new("time 4/4\nnote 1/8\ngrouping 2+2\nSD | d ||.\n");
        let doc = p.parse().unwrap();
        let score = normalize::normalize_document(&doc);
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].barline.as_deref(), Some("double"));
        assert_eq!(score.measures[0].closing_barline.as_deref(), Some("double"));
        assert!(score.measures[0].volta_terminator);
    }

    #[test]
    fn test_repeat_both_closes_previous_span_and_opens_next() {
        let measures = vec![
            normalize::NormalizedMeasure {
                index: 0,
                global_index: 0,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![],
                barline: Some("repeat-start".into()),
                closing_barline: None,
                start_nav: None,
                end_nav: None,
                volta: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            normalize::NormalizedMeasure {
                index: 1,
                global_index: 1,
                paragraph_index: 0,
                measure_in_paragraph: 1,
                source_line: 1,
                events: vec![],
                barline: Some("repeat-both".into()),
                closing_barline: None,
                start_nav: None,
                end_nav: None,
                volta: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            normalize::NormalizedMeasure {
                index: 2,
                global_index: 2,
                paragraph_index: 0,
                measure_in_paragraph: 2,
                source_line: 1,
                events: vec![],
                barline: Some("repeat-end".into()),
                closing_barline: None,
                start_nav: None,
                end_nav: None,
                volta: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
        ];

        let mut errors = Vec::new();
        let spans = normalize::derive_repeat_spans(&measures, &mut errors);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].start_measure, 0);
        assert_eq!(spans[0].end_measure, 1);
        assert_eq!(spans[1].start_measure, 1);
        assert_eq!(spans[1].end_measure, 2);
    }
}
