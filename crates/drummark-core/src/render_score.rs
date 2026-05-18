use crate::event::EventKind;
use crate::fraction::Fraction;
use crate::hairpin::HairpinKind;
use crate::nav::{EndNav, StartNav};
use crate::normalize;

pub fn derive_render_score(score: &normalize::NormalizedScore) -> drummark_layout::RenderScore {
    let default_voice_tracks = default_voice_tracks(score);
    drummark_layout::RenderScore {
        version: drummark_layout::RENDER_SCORE_VERSION.to_string(),
        header: drummark_layout::RenderHeader {
            tempo: score.header.tempo,
            time_beats: score.header.time_beats,
            time_beat_unit: score.header.time_beat_unit,
            divisions: score.header.divisions,
            note_value: score.header.note_value,
            grouping: score.header.grouping.clone(),
            title: score.header.title.clone(),
            subtitle: score.header.subtitle.clone(),
            composer: score.header.composer.clone(),
        },
        tracks: score
            .tracks
            .iter()
            .map(|track| drummark_layout::RenderTrack {
                id: track.id.clone(),
                family: track.family.clone(),
            })
            .collect(),
        measures: score
            .measures
            .iter()
            .map(|measure| {
                let mut events: Vec<drummark_layout::RenderEvent> = measure
                    .events
                    .iter()
                    .map(event_to_render_event)
                    .collect();

                if measure.measure_repeat_slashes.is_none() && measure.multi_rest_count.is_none() {
                    events.extend(derive_implicit_rest_events(score, measure, &default_voice_tracks));
                    sort_render_events(&mut events);
                }

                drummark_layout::RenderMeasure {
                    index: measure.index,
                    global_index: measure.global_index,
                    paragraph_index: measure.paragraph_index,
                    measure_in_paragraph: measure.measure_in_paragraph,
                    source_line: measure.source_line,
                    events,
                    barline: measure.barline.clone(),
                    closing_barline: measure.closing_barline.clone(),
                    start_nav: measure.start_nav.as_ref().map(start_nav_kind),
                    end_nav: measure.end_nav.as_ref().map(end_nav_kind),
                    volta_indices: measure.volta.clone(),
                    hairpins: measure
                        .hairpins
                        .iter()
                        .map(|hairpin| drummark_layout::HairpinSpan {
                            kind: match hairpin.kind {
                                HairpinKind::Crescendo => drummark_layout::HairpinKind::Crescendo,
                                HairpinKind::Decrescendo => drummark_layout::HairpinKind::Decrescendo,
                            },
                            start: drummark_layout::Fraction {
                                numerator: hairpin.start.numerator as u32,
                                denominator: hairpin.start.denominator as u32,
                            },
                            end: drummark_layout::Fraction {
                                numerator: hairpin.end.numerator as u32,
                                denominator: hairpin.end.denominator as u32,
                            },
                            start_measure_index: hairpin.start_measure_index as u32,
                            end_measure_index: hairpin.end_measure_index as u32,
                        })
                        .collect(),
                    measure_repeat_slashes: measure.measure_repeat_slashes,
                    multi_rest_count: measure.multi_rest_count,
                    note_value: measure.note_value,
                    volta_terminator: measure.volta_terminator,
                }
            })
            .collect(),
        errors: score.errors.clone(),
        repeat_spans: score
            .repeat_spans
            .iter()
            .map(|span| drummark_layout::RepeatSpan {
                start_measure: span.start_measure,
                end_measure: span.end_measure,
                times: span.times,
            })
            .collect(),
    }
}

fn event_to_render_event(event: &crate::event::NormalizedEvent) -> drummark_layout::RenderEvent {
    drummark_layout::RenderEvent {
        track: event.track.clone(),
        track_family: crate::resolve::get_track_family(&event.track).as_str().to_string(),
        start: render_fraction(event.start),
        duration: render_fraction(event.duration),
        kind: match event.kind {
            EventKind::Hit => drummark_layout::EventKind::Hit,
            EventKind::Rest => drummark_layout::EventKind::Rest,
            EventKind::Sticking => drummark_layout::EventKind::Sticking,
        },
        glyph: event.glyph.clone(),
        modifiers: event.modifiers.clone(),
        modifier: event.modifier.clone(),
        voice: event.voice,
        beam: event.beam.clone(),
        tuplet: event.tuplet,
    }
}

fn render_fraction(fraction: Fraction) -> drummark_layout::Fraction {
    drummark_layout::Fraction {
        numerator: fraction.numerator as u32,
        denominator: fraction.denominator as u32,
    }
}

fn default_voice_tracks(score: &normalize::NormalizedScore) -> [(String, String); 2] {
    let mut voice_one = ("HH".to_string(), "cymbal".to_string());
    let mut voice_two = ("BD".to_string(), "drum".to_string());
    for track in &score.tracks {
        let family = crate::resolve::get_track_family(&track.id).as_str().to_string();
        match crate::resolve::voice_for_track(&track.id) {
            1 if voice_one.0 == "HH" => voice_one = (track.id.clone(), family),
            2 if voice_two.0 == "BD" => voice_two = (track.id.clone(), family),
            _ => {}
        }
    }
    [voice_one, voice_two]
}

fn derive_implicit_rest_events(
    score: &normalize::NormalizedScore,
    measure: &normalize::NormalizedMeasure,
    default_voice_tracks: &[(String, String); 2],
) -> Vec<drummark_layout::RenderEvent> {
    let measure_duration = Fraction::new(
        score.header.time_beats as u64,
        score.header.time_beat_unit as u64,
    );
    let grouping = if score.header.grouping.is_empty() {
        vec![score.header.time_beats]
    } else {
        score.header.grouping.clone()
    };

    let mut rests = Vec::new();
    let active_voices = active_voices_for_score(score);
    for voice in [1_u8, 2_u8] {
        if !active_voices.contains(&voice) {
            continue;
        }
        let mut voice_events: Vec<&crate::event::NormalizedEvent> = measure
            .events
            .iter()
            .filter(|event| event.voice == voice && event.kind != EventKind::Sticking)
            .collect();
        voice_events.sort_by(|left, right| left.start.compare(right.start));

        let fallback = &default_voice_tracks[(voice - 1) as usize];
        let track = voice_events
            .first()
            .map(|event| (event.track.clone(), crate::resolve::get_track_family(&event.track).as_str().to_string()))
            .unwrap_or_else(|| fallback.clone());

        if voice_events.is_empty() {
            push_rest_event(&mut rests, Fraction::zero(), measure_duration, voice, &track.0, &track.1);
            continue;
        }

        let mut cursor = Fraction::zero();
        for event in voice_events {
            if event.start.compare(cursor).is_gt() {
                extend_voice_rests(
                    &mut rests,
                    cursor,
                    event.start,
                    voice,
                    &track.0,
                    &track.1,
                    &grouping,
                    score.header.time_beat_unit,
                );
            }
            let event_end = event.start.add(event.duration);
            if event_end.compare(cursor).is_gt() {
                cursor = event_end;
            }
        }
        if cursor.compare(measure_duration).is_lt() {
            extend_voice_rests(
                &mut rests,
                cursor,
                measure_duration,
                voice,
                &track.0,
                &track.1,
                &grouping,
                score.header.time_beat_unit,
            );
        }
    }

    rests
}

fn active_voices_for_score(score: &normalize::NormalizedScore) -> Vec<u8> {
    let mut voices = Vec::new();
    for track in &score.tracks {
        let voice = crate::resolve::voice_for_track(&track.id);
        if !voices.contains(&voice) {
            voices.push(voice);
        }
    }
    if voices.is_empty() {
        // Anonymous-only scores have no registered tracks; default to voice 1.
        voices.push(1);
    }
    voices
}

fn extend_voice_rests(
    output: &mut Vec<drummark_layout::RenderEvent>,
    start: Fraction,
    end: Fraction,
    voice: u8,
    track: &str,
    track_family: &str,
    grouping: &[u32],
    beat_unit: u32,
) {
    let mut cursor = start;
    for boundary in grouping_boundaries(grouping, beat_unit) {
        if cursor.compare(boundary).is_ge() {
            continue;
        }
        if end.compare(boundary).is_le() {
            push_rest_span(output, cursor, end.subtract(cursor), voice, track, track_family);
            return;
        }
        push_rest_span(output, cursor, boundary.subtract(cursor), voice, track, track_family);
        cursor = boundary;
    }
    if cursor.compare(end).is_lt() {
        push_rest_span(output, cursor, end.subtract(cursor), voice, track, track_family);
    }
}

fn grouping_boundaries(grouping: &[u32], beat_unit: u32) -> Vec<Fraction> {
    let mut boundaries = Vec::with_capacity(grouping.len());
    let mut accumulated = 0_u64;
    for group in grouping {
        accumulated += *group as u64;
        boundaries.push(Fraction::new(accumulated, beat_unit as u64));
    }
    boundaries
}

fn push_rest_event(
    output: &mut Vec<drummark_layout::RenderEvent>,
    start: Fraction,
    duration: Fraction,
    voice: u8,
    track: &str,
    track_family: &str,
) {
    if duration.is_zero() {
        return;
    }
    output.push(drummark_layout::RenderEvent {
        track: track.to_string(),
        track_family: track_family.to_string(),
        start: render_fraction(start),
        duration: render_fraction(duration),
        kind: drummark_layout::EventKind::Rest,
        glyph: "-".to_string(),
        modifiers: Vec::new(),
        modifier: None,
        voice,
        beam: "none".to_string(),
        tuplet: None,
    });
}

fn push_rest_span(
    output: &mut Vec<drummark_layout::RenderEvent>,
    start: Fraction,
    duration: Fraction,
    voice: u8,
    track: &str,
    track_family: &str,
) {
    if duration.is_zero() {
        return;
    }

    let mut cursor = start;
    let mut remaining = duration;
    for primitive in decompose_rest_duration(duration) {
        push_rest_event(output, cursor, primitive, voice, track, track_family);
        cursor = cursor.add(primitive);
        remaining = remaining.subtract(primitive);
    }

    if !remaining.is_zero() {
        push_rest_event(output, cursor, remaining, voice, track, track_family);
    }
}

fn decompose_rest_duration(duration: Fraction) -> Vec<Fraction> {
    let primitives = [
        Fraction::one(),
        Fraction::new(1, 2),
        Fraction::new(1, 4),
        Fraction::new(1, 8),
        Fraction::new(1, 16),
        Fraction::new(1, 32),
    ];

    let mut result = Vec::new();
    let mut remaining = duration;
    for primitive in primitives {
        while remaining.compare(primitive).is_ge() {
            result.push(primitive);
            remaining = remaining.subtract(primitive);
        }
    }
    result
}

fn sort_render_events(events: &mut [drummark_layout::RenderEvent]) {
    events.sort_by(|left, right| {
        let start = (left.start.numerator as u64 * right.start.denominator as u64)
            .cmp(&(right.start.numerator as u64 * left.start.denominator as u64));
        if start != std::cmp::Ordering::Equal {
            return start;
        }
        let kind_rank = |kind: &drummark_layout::EventKind| match kind {
            drummark_layout::EventKind::Hit => 0_u8,
            drummark_layout::EventKind::Sticking => 1_u8,
            drummark_layout::EventKind::Rest => 2_u8,
        };
        let kind = kind_rank(&left.kind).cmp(&kind_rank(&right.kind));
        if kind != std::cmp::Ordering::Equal {
            return kind;
        }
        let duration = (left.duration.numerator as u64 * right.duration.denominator as u64)
            .cmp(&(right.duration.numerator as u64 * left.duration.denominator as u64));
        if duration != std::cmp::Ordering::Equal {
            return duration.reverse();
        }
        left.voice.cmp(&right.voice)
    });
}

fn start_nav_kind(nav: &StartNav) -> drummark_layout::NavMarker {
    match nav {
        StartNav::Segno { .. } => drummark_layout::NavMarker::Segno,
        StartNav::Coda { .. } => drummark_layout::NavMarker::Coda,
    }
}

fn end_nav_kind(nav: &EndNav) -> drummark_layout::NavJump {
    match nav {
        EndNav::Fine { .. } => drummark_layout::NavJump::Fine,
        EndNav::DC { .. } => drummark_layout::NavJump::DC,
        EndNav::DS { .. } => drummark_layout::NavJump::DS,
        EndNav::DCalFine { .. } => drummark_layout::NavJump::DCalFine,
        EndNav::DCalCoda { .. } => drummark_layout::NavJump::DCalCoda,
        EndNav::DSalFine { .. } => drummark_layout::NavJump::DSalFine,
        EndNav::DSalCoda { .. } => drummark_layout::NavJump::DSalCoda,
        EndNav::ToCoda { .. } => drummark_layout::NavJump::ToCoda,
    }
}

trait TrackFamilyExt {
    fn as_str(&self) -> &'static str;
}

impl TrackFamilyExt for crate::resolve::TrackFamily {
    fn as_str(&self) -> &'static str {
        match self {
            crate::resolve::TrackFamily::Cymbal => "cymbal",
            crate::resolve::TrackFamily::Drum => "drum",
            crate::resolve::TrackFamily::Pedal => "pedal",
            crate::resolve::TrackFamily::Percussion => "percussion",
            crate::resolve::TrackFamily::Auxiliary => "auxiliary",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::derive_render_score;
    use crate::fraction::Fraction;
    use crate::hairpin::{HairpinIntent, HairpinKind};
    use crate::nav::{Anchor, EndNav, StartNav};
    use crate::parser::Parser;

    #[test]
    fn derives_render_score_from_normalized_score() {
        let source = "title Smoke\ntime 4/4\nHH | x - x - |\nSD | - o - o |\n";
        let document = Parser::new(source).parse().expect("parse");
        let normalized = crate::normalize::normalize_document(&document);
        let render = derive_render_score(&normalized);

        assert_eq!(render.version, drummark_layout::RENDER_SCORE_VERSION);
        assert_eq!(render.header.time_beats, 4);
        assert_eq!(render.measures.len(), 1);
        assert_eq!(render.measures[0].source_line, normalized.measures[0].source_line);
        assert_eq!(render.measures[0].events[0].track_family, "cymbal");
    }

    #[test]
    fn preserves_navigation_and_hairpin_semantics() {
        let score = crate::normalize::NormalizedScore {
            version: "1".into(),
            header: crate::normalize::NormalizedHeader {
                title: None,
                subtitle: None,
                composer: None,
                tempo: 120,
                time_beats: 4,
                time_beat_unit: 4,
                divisions: 16,
                note_value: 8,
                grouping: vec![1, 1, 1, 1],
            },
            tracks: vec![crate::normalize::NormalizedTrack {
                id: "HH".into(),
                family: "cymbal".into(),
            }],
            measures: vec![crate::normalize::NormalizedMeasure {
                index: 7,
                global_index: 7,
                paragraph_index: 0,
                measure_in_paragraph: 7,
                source_line: 12,
                events: vec![],
                barline: Some("double".into()),
                closing_barline: Some("double".into()),
                start_nav: Some(StartNav::Segno { anchor: Anchor::LeftEdge }),
                end_nav: Some(EndNav::DCalCoda { anchor: Anchor::RightEdge }),
                volta: Some(vec![1, 2]),
                hairpins: vec![HairpinIntent {
                    kind: HairpinKind::Crescendo,
                    start: Fraction { numerator: 0, denominator: 1 },
                    start_measure_index: 7,
                    end: Fraction { numerator: 1, denominator: 1 },
                    end_measure_index: 8,
                }],
                measure_repeat_slashes: Some(1),
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: true,
            }],
            errors: vec![],
            repeat_spans: vec![crate::normalize::RepeatSpan {
                start_measure: 7,
                end_measure: 8,
                times: 2,
            }],
        };

        let render = derive_render_score(&score);
        assert!(matches!(render.measures[0].start_nav, Some(drummark_layout::NavMarker::Segno)));
        assert!(matches!(render.measures[0].end_nav, Some(drummark_layout::NavJump::DCalCoda)));
        assert_eq!(render.measures[0].hairpins.len(), 1);
        assert_eq!(render.measures[0].hairpins[0].start_measure_index, 7);
        assert_eq!(render.repeat_spans[0].times, 2);
    }

    #[test]
    fn derives_implicit_voice_rests_for_measure_gaps() {
        let source = "time 4/4\nnote 1/8\ngrouping 2+2\nHH | x x x x x x x x |\nBD | p - - - p - - - |\n";
        let document = Parser::new(source).parse().expect("parse");
        let normalized = crate::normalize::normalize_document(&document);
        let render = derive_render_score(&normalized);
        let measure = &render.measures[0];
        let rest_events: Vec<_> = measure.events.iter().filter(|event| event.kind == drummark_layout::EventKind::Rest).collect();
        assert_eq!(rest_events.len(), 4, "3/8 gaps should decompose into quarter + eighth twice");
        assert!(rest_events.iter().all(|event| event.voice == 2));
        assert!(rest_events.iter().any(|event| event.duration == drummark_layout::Fraction { numerator: 1, denominator: 4 }));
        assert!(rest_events.iter().any(|event| event.duration == drummark_layout::Fraction { numerator: 1, denominator: 8 }));
    }

    #[test]
    fn does_not_invent_missing_voice_rests() {
        let source = "time 4/4\ndivisions 4\ngrouping 2+2\nBD | b - - - |\n";
        let document = Parser::new(source).parse().expect("parse");
        let normalized = crate::normalize::normalize_document(&document);
        let render = derive_render_score(&normalized);
        let measure = &render.measures[0];
        let rest_events: Vec<_> = measure.events.iter().filter(|event| event.kind == drummark_layout::EventKind::Rest).collect();
        assert!(!measure.events.iter().any(|event| {
            event.kind == drummark_layout::EventKind::Rest && event.voice == 1
        }));
        assert_eq!(rest_events.len(), 3);
        assert!(rest_events.iter().any(|event| event.duration == drummark_layout::Fraction { numerator: 1, denominator: 4 }));
        assert!(rest_events.iter().any(|event| event.duration == drummark_layout::Fraction { numerator: 1, denominator: 8 }));
        assert!(rest_events.iter().any(|event| event.duration == drummark_layout::Fraction { numerator: 1, denominator: 2 }));
    }
}
