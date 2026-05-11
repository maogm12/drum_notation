#[derive(Debug)]
pub struct Document {
    pub headers: HeaderSection,
    pub paragraphs: Vec<Paragraph>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct Paragraph {
    pub note: Option<(u32, u32)>,
    pub lines: Vec<TrackLine>,
}

#[derive(Debug)]
pub struct TrackLine {
    pub track: Option<String>,
    pub measures: Vec<MeasureSection>,
}

#[derive(Debug)]
pub struct MeasureSection {
    pub barline: Barline,
    pub tokens: Vec<MeasureExpr>,
    pub closing_barline: Option<Barline>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MeasureExpr {
    BasicNote(NoteExpr),
    SummonedNote { track: String, note: NoteExpr },
    RoutedBracedBlock { track: String, content: Vec<MeasureExpr> },
    InlineBracedBlock(Vec<MeasureExpr>),
    Group(GroupExpr),
    CombinedHit(Vec<MeasureExpr>),
    MeasureRepeat(u32),
    MultiRest(u32),
    InlineRepeat(u32),
    Crescendo,
    Decrescendo,
    HairpinEnd,
    NavMarker(String),
    NavJump(String),
}

#[derive(Debug, Clone)]
pub struct NoteExpr {
    pub glyph: String,
    pub dots: u32,
    pub halves: u32,
    pub stars: u32,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone)]
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
