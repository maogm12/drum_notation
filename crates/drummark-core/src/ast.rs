pub struct Document {
    pub headers: HeaderSection,
    pub paragraphs: Vec<Paragraph>,
    pub errors: Vec<ParseError>,
}

pub struct HeaderSection {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub composer: Option<String>,
    pub tempo: Option<u32>,
    pub time: Option<(u32, u32)>,
    pub grouping: Option<Vec<u32>>,
    pub note: Option<(u32, u32)>,
    pub divisions: Option<u32>,
}

pub struct Paragraph {
    pub note: Option<(u32, u32)>,
    pub lines: Vec<TrackLine>,
}

pub struct TrackLine {
    pub track: Option<String>,
    pub measures: Vec<MeasureSection>,
}

pub struct MeasureSection {
    pub barline: Barline,
    pub tokens: Vec<MeasureExpr>,
}

pub enum Barline {
    Regular,
    Double,
    RepeatStart,
    RepeatEnd,
    VoltaTerminator,
    DoubleVoltaTerminator,
    VoltaRepeatStart,
    Volta { prefix: String, numbers: Vec<u32> },
}

pub enum MeasureExpr {
    BasicNote(NoteExpr),
    Group(GroupExpr),
    CombinedHit(Vec<NoteExpr>),
    MeasureRepeat(u32),
    MultiRest(u32),
    InlineRepeat(u32),
    Crescendo,
    Decrescendo,
    HairpinEnd,
    NavMarker(String),
    NavJump(String),
}

pub struct NoteExpr {
    pub glyph: String,
    pub dots: u32,
    pub halves: u32,
    pub stars: u32,
    pub modifiers: Vec<String>,
}

pub struct GroupExpr {
    pub n: Option<u32>,
    pub items: Vec<MeasureExpr>,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: u32,
    pub column: u32,
    pub message: String,
}
