use crate::fraction::Fraction;

#[derive(Debug, Clone)]
pub enum StartNav {
    Segno { anchor: Anchor },
    Coda { anchor: Anchor },
}

#[derive(Debug, Clone)]
pub enum EndNav {
    Fine { anchor: Anchor },
    DC { anchor: Anchor },
    DS { anchor: Anchor },
    DCalFine { anchor: Anchor },
    DCalCoda { anchor: Anchor },
    DSalFine { anchor: Anchor },
    DSalCoda { anchor: Anchor },
    ToCoda { anchor: Anchor },
}

#[derive(Debug, Clone)]
pub enum Anchor {
    LeftEdge,
    RightEdge,
    EventBefore(Fraction),
    EventAfter(Fraction),
}

impl EndNav {
    pub fn kind_name(&self) -> &str {
        match self {
            EndNav::Fine { .. } => "fine",
            EndNav::DC { .. } => "dc",
            EndNav::DS { .. } => "ds",
            EndNav::DCalFine { .. } => "dc-al-fine",
            EndNav::DCalCoda { .. } => "dc-al-coda",
            EndNav::DSalFine { .. } => "ds-al-fine",
            EndNav::DSalCoda { .. } => "ds-al-coda",
            EndNav::ToCoda { .. } => "to-coda",
        }
    }

    /// Returns the barline type forced by this end navigation.
    /// `fine` → "final"; `dc`/`ds` family → "double".
    pub fn forced_barline(&self) -> BarlineType {
        match self {
            EndNav::Fine { .. } => BarlineType::Final,
            _ => BarlineType::Double,
        }
    }
}

impl StartNav {
    pub fn kind_name(&self) -> &str {
        match self {
            StartNav::Segno { .. } => "segno",
            StartNav::Coda { .. } => "coda",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarlineType {
    Regular,
    Double,
    Final,
    RepeatStart,
    RepeatEnd,
    RepeatBoth,
}
