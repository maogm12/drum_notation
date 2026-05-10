#[derive(Debug, Clone)]
pub struct VoltaIntent {
    pub indices: Vec<u32>,
}

/// Propagate volta seeds forward through measures.
/// A measure with `volta_indices` seeds an active volta that continues
/// until cleared by a repeat-end, repeat-both, or voltaTerminator.
pub fn propagate_voltas(
    measures: &mut [VoltaMeasure],
) {
    let mut active: Option<Vec<u32>> = None;

    for m in measures.iter_mut() {
        // Seed: this measure has its own volta indices
        if let Some(ref indices) = m.seed_volta {
            active = Some(indices.clone());
        }

        // Apply active volta
        m.volta = active.clone().map(|v| VoltaIntent { indices: v });

        // Clear: repeat-end, repeat-both, or volta terminator
        if m.repeat_end || m.repeat_both || m.volta_terminator {
            active = None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoltaMeasure {
    pub seed_volta: Option<Vec<u32>>,
    pub volta: Option<VoltaIntent>,
    pub repeat_end: bool,
    pub repeat_both: bool,
    pub volta_terminator: bool,
}
