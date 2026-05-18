use crate::fraction::Fraction;

#[derive(Debug, Clone)]
pub struct HairpinIntent {
    pub kind: HairpinKind,
    pub start: Fraction,
    pub start_measure_index: usize,
    pub end: Fraction,
    pub end_measure_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HairpinKind {
    Crescendo,
    Decrescendo,
}

#[derive(Debug, Clone)]
pub struct HairpinState {
    pub active_type: Option<HairpinKind>,
    pub active_start: Option<Fraction>,
    pub start_measure_index: usize,
}

impl HairpinState {
    pub fn new() -> Self {
        Self {
            active_type: None,
            active_start: None,
            start_measure_index: 0,
        }
    }
}

impl Default for HairpinState {
    fn default() -> Self {
        Self::new()
    }
}

/// Process events in a single measure for a single track, collecting hairpins.
pub fn collect_track_hairpins(
    events: &[(Fraction, Option<HairpinKind>, Option<()>)], // (start, open_kind, close_signal)
    measure_index: usize,
    state: &mut HairpinState,
) -> Vec<HairpinIntent> {
    let mut hairpins = Vec::new();

    for &(start, open_kind, close_signal) in events {
        if let Some(kind) = open_kind {
            // Close any existing hairpin first
            if let Some(active) = state.active_type.take() {
                if let Some(active_start) = state.active_start.take() {
                    hairpins.push(HairpinIntent {
                        kind: active,
                        start: active_start,
                        start_measure_index: state.start_measure_index,
                        end: start,
                        end_measure_index: measure_index,
                    });
                }
            }
            // Open new hairpin
            state.active_type = Some(kind);
            state.active_start = Some(start);
            state.start_measure_index = measure_index;
        }

        if close_signal.is_some() {
            if let Some(active) = state.active_type.take() {
                if let Some(active_start) = state.active_start.take() {
                    hairpins.push(HairpinIntent {
                        kind: active,
                        start: active_start,
                        start_measure_index: state.start_measure_index,
                        end: start,
                        end_measure_index: measure_index,
                    });
                }
            }
        }
    }

    hairpins
}

/// Close any dangling hairpin at the end of the score.
pub fn close_dangling_hairpin(
    state: &mut HairpinState,
    last_measure_index: usize,
    end_position: Fraction,
) -> Option<HairpinIntent> {
    if let Some(active) = state.active_type.take() {
        if let Some(active_start) = state.active_start.take() {
            return Some(HairpinIntent {
                kind: active,
                start: active_start,
                start_measure_index: state.start_measure_index,
                end: end_position,
                end_measure_index: last_measure_index,
            });
        }
    }
    None
}
