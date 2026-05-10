use crate::fraction::{Fraction, fractions_equal, calculate_token_weight_as_fraction};
use crate::resolve::{get_track_family, TrackFamily};
use crate::validate::{validate_modifier_legality, validate_grouping};
use crate::hairpin::{HairpinState, HairpinIntent, HairpinKind, collect_track_hairpins, close_dangling_hairpin};
use crate::nav::{StartNav, EndNav, Anchor, BarlineType};
use crate::volta::{VoltaMeasure, propagate_voltas};
use crate::event::{NormalizedEvent, EventKind, TokenGlyph, token_to_events, scan_hairpin_tokens};
use crate::ast::{Document, Barline, MeasureExpr, NoteExpr, GroupExpr, MeasureSection, TrackLine, HeaderSection};
use std::collections::{HashMap, HashSet};

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
    pub start_nav: Option<StartNav>,
    pub end_nav: Option<EndNav>,
    pub volta: Option<Vec<u32>>,
    pub hairpins: Vec<HairpinIntent>,
    pub measure_repeat_slashes: Option<u32>,
    pub multi_rest_count: Option<u32>,
    pub note_value: u32,
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
            items: hits.iter().map(|n| TokenGlyph::Basic {
                value: n.glyph.clone(),
                dots: n.dots,
                halves: n.halves,
                stars: n.stars,
                modifiers: n.modifiers.clone(),
                track_override: None,
            }).collect(),
        },
        MeasureExpr::Group(g) => TokenGlyph::Group {
            count: g.n.unwrap_or(0),
            span: g.items.len() as u32,
            items: g.items.iter().map(to_token_glyph).collect(),
            modifiers: g.modifiers.clone(),
        },
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
        MeasureExpr::MeasureRepeat(count) => TokenGlyph::Basic {
            value: "-".to_string(), dots: 0, halves: 0, stars: 0,
            modifiers: vec![], track_override: None,
        },
        MeasureExpr::MultiRest(count) => TokenGlyph::Basic {
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
        Barline::VoltaTerminator => None, // handled via volta data
        Barline::DoubleVoltaTerminator => None,
        Barline::VoltaRepeatStart => Some("repeat-start".to_string()),
        Barline::Volta { prefix, numbers } => {
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
    let mut repeat_spans: Vec<RepeatSpan> = Vec::new();

    // Per-track hairpin state
    let mut hairpin_states: HashMap<String, HairpinState> = HashMap::new();

    for (para_idx, para) in doc.paragraphs.iter().enumerate() {
        let para_note_value = para.note.map(|(_, d)| d).unwrap_or(note_value);

        // Determine measure count from first line
        let measure_count = para.lines.first()
            .map(|l| l.measures.len() as u32)
            .unwrap_or(0);

        for m_idx in 0..measure_count as usize {
            let mut measure_events: Vec<NormalizedEvent> = Vec::new();
            let mut measure_hairpins: Vec<HairpinIntent> = Vec::new();
            let mut barline: Option<String> = None;
            let mut repeat_start = false;
            let mut repeat_end = false;
            let mut volta_indices: Option<Vec<u32>> = None;
            let mut volta_terminator = false;
            let mut measure_repeat_slashes: Option<u32> = None;
            let mut multi_rest_count: Option<u32> = None;
            let mut start_nav: Option<StartNav> = None;
            let mut end_nav: Option<EndNav> = None;

            for line in &para.lines {
                if m_idx >= line.measures.len() { continue; }
                let ms = &line.measures[m_idx];
                let context_track = line.track.as_deref();
                let use_track = context_track.unwrap_or("ANONYMOUS");

                // Barline metadata from first line
                if barline.is_none() {
                    barline = barline_type(&ms.barline);
                }

                // Check for repeat/volta metadata
                match &ms.barline {
                    Barline::RepeatStart | Barline::VoltaRepeatStart => repeat_start = true,
                    Barline::RepeatEnd => repeat_end = true,
                    Barline::Volta { numbers, .. } => {
                        volta_indices = Some(numbers.clone());
                    }
                    Barline::VoltaTerminator | Barline::DoubleVoltaTerminator => {
                        volta_terminator = true;
                    }
                    _ => {}
                }

                // Scan tokens
                let mut tokens: Vec<TokenGlyph> = ms.tokens.iter()
                    .map(to_token_glyph)
                    .collect();

                // Extract non-display tokens
                tokens.retain(|t| {
                    match t {
                        TokenGlyph::Basic { value, .. } if value == "-" => true,
                        _ => true,
                    }
                });

                // Scan for measure-repeat, multi-rest, nav markers
                for tok in &ms.tokens {
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

                // Convert tokens to events
                let duration_per_quarter = Fraction::new(1, para_note_value as u64);
                let duration: Fraction = Fraction::zero();

                // Calculate total weight for measure duration
                let mut total_weight = Fraction::zero();
                for t in &tokens {
                    total_weight = total_weight.add(token_weight(t));
                }
                let measure_duration = total_weight.multiply(duration_per_quarter);

                // Expand tokens to events
                let mut position = Fraction::zero();
                for (t_idx, token) in tokens.iter().enumerate() {
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
                        position,
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

                // Collect hairpins per track
                let hairpin_scan = scan_hairpin_tokens(&tokens, Fraction::zero(), divisions);
                let state = hairpin_states.entry(use_track.to_string())
                    .or_insert_with(HairpinState::new);
                let track_hairpins = collect_track_hairpins(&hairpin_scan, global_index as usize, state);
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
                start_nav,
                end_nav,
                volta: volta_indices,
                hairpins: measure_hairpins,
                measure_repeat_slashes,
                multi_rest_count,
                note_value: para_note_value,
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
        volta_terminator: m.barline.is_none() && m.volta.is_some(),
    }).collect();

    propagate_voltas(&mut volta_measures);

    for (i, vm) in volta_measures.iter().enumerate() {
        all_measures[i].volta = vm.volta.as_ref().map(|v| v.indices.clone());
    }

    // ── Post-pass 2: Close dangling hairpins ─────────────────────

    let last_idx = all_measures.len().saturating_sub(1);
    for (track_id, state) in hairpin_states.iter_mut() {
        if let Some(hairpin) = close_dangling_hairpin(state, last_idx, Fraction::new(1, 1)) {
            if let Some(m) = all_measures.last_mut() {
                m.hairpins.push(hairpin);
            }
        }
    }

    // ── Post-pass 3: Final barline ───────────────────────────────

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

fn token_weight(token: &TokenGlyph) -> Fraction {
    match token {
        TokenGlyph::Basic { dots, halves, stars, .. } => {
            calculate_token_weight_as_fraction(*dots, *stars, *halves, None)
        }
        TokenGlyph::Group { span, items, .. } => {
            let w: Fraction = items.iter().map(token_weight).fold(Fraction::zero(), |a, b| a.add(b));
            if w.is_zero() { Fraction::new(*span as u64, 1) } else { w }
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
