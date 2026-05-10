use logos::Logos;
use crate::lexer::Token;
use crate::ast::*;

pub struct Parser<'a> {
    lexer: logos::Lexer<'a, Token>,
    peek_buf: Vec<Token>,
    errors: Vec<ParseError>,
    source: &'a str,
    last_end: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Token::lexer(source),
            peek_buf: Vec::new(),
            errors: Vec::new(),
            source,
            last_end: 0,
        }
    }

    pub fn parse(mut self) -> Result<Document, Vec<ParseError>> {
        let doc = self.parse_document();
        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        Ok(doc)
    }

    // ── Token Access ──────────────────────────────────────────────

    fn peek(&mut self) -> Option<Token> {
        self.peek_n(0)
    }

    fn peek_raw(&mut self) -> Option<Token> {
        if !self.peek_buf.is_empty() {
            return Some(self.peek_buf[0].clone());
        }
        let mut iter = self.lexer.clone();
        loop {
            match iter.next() {
                Some(Ok(t)) => return Some(t),
                Some(Err(())) => {
                    let span = iter.span();
                    let ch = &self.source[span.start..span.end];
                    return Some(Token::FreeText(ch.to_string()));
                }
                None => return None,
            }
        }
    }

    fn peek_n(&mut self, n: usize) -> Option<Token> {
        while self.peek_buf.len() <= n {
            match self.lexer.next() {
                Some(Ok(Token::Space | Token::Comment)) => continue,
                Some(Ok(t)) => self.peek_buf.push(t),
                Some(Err(())) => {
                    let span = self.lexer.span();
                    let start = span.start;
                    let mut end = span.end;
                    while let Some(Err(())) = self.lexer.next() {
                        end = self.lexer.span().end;
                    }
                    let text = &self.source[start..end];
                    self.peek_buf.push(Token::FreeText(text.to_string()));
                }
                None => return None,
            }
        }
        Some(self.peek_buf[n].clone())
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        let t = if !self.peek_buf.is_empty() {
            self.peek_buf.remove(0)
        } else {
            loop {
                match self.lexer.next() {
                    Some(Ok(Token::Space | Token::Comment)) => continue,
                    Some(Ok(t)) => break t,
                    Some(Err(())) => {
                        let span = self.lexer.span();
                        let start = span.start;
                        let mut end = span.end;
                        while let Some(Err(())) = self.lexer.next() {
                            end = self.lexer.span().end;
                        }
                        let text = &self.source[start..end];
                        break Token::FreeText(text.to_string());
                    }
                    None => return Err(self.error_at(self.last_end, "unexpected end of input")),
                }
            }
        };
        self.last_end = self.lexer.span().end;
        Ok(t)
    }

    fn next_raw(&mut self) -> Result<Token, ParseError> {
        if !self.peek_buf.is_empty() {
            self.peek_buf.clear();
        }
        match self.lexer.next() {
            Some(Ok(t)) => {
                self.last_end = self.lexer.span().end;
                Ok(t)
            }
            Some(Err(())) => {
                let span = self.lexer.span();
                let s = &self.source[span.start..span.end];
                self.last_end = span.end;
                Ok(Token::FreeText(s.to_string()))
            }
            None => Err(self.error_at(self.last_end, "unexpected end of input")),
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == Some(Token::Newline) {
            self.next().ok();
        }
    }

    fn skip_trivia(&mut self) {
        while matches!(self.peek_raw(), Some(Token::Space | Token::Comment)) {
            self.next_raw().ok();
        }
    }

    fn expect(&mut self, expected: Token) -> Result<Token, ParseError> {
        let t = self.next()?;
        if std::mem::discriminant(&t) == std::mem::discriminant(&expected) {
            Ok(t)
        } else {
            Err(self.error_at(self.last_end, &format!("expected {:?}, found {:?}", expected, t)))
        }
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn error_at(&mut self, pos: usize, msg: &str) -> ParseError {
        let line_col = self.line_column(pos);
        ParseError { line: line_col.0, column: line_col.1, message: msg.to_string() }
    }

    fn push_error(&mut self, pos: usize, msg: &str) {
        let e = self.error_at(pos, msg);
        self.errors.push(e);
    }

    fn line_column(&self, offset: usize) -> (u32, u32) {
        let offset = if offset > self.source.len() { self.source.len() } else { offset };
        let prefix = &self.source[..offset];
        let line = prefix.bytes().filter(|&b| b == b'\n').count() as u32 + 1;
        let last_nl = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = (offset - last_nl + 1) as u32;
        (line, column)
    }

    fn token_text(&self, t: &Token) -> String {
        match t {
            Token::FreeText(s) => s.clone(),
            _ => format!("{:?}", t),
        }
    }

    fn expect_integer(&mut self) -> Result<u32, ParseError> {
        match self.next()? {
            Token::Integer(n) => Ok(n),
            t => Err(self.error_at(self.last_end, &format!("expected integer, found {:?}", t))),
        }
    }

    fn extract_multi_rest_count(&self, s: &str) -> u32 {
        s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(1)
    }

    // ── Document ──────────────────────────────────────────────────

    fn parse_document(&mut self) -> Document {
        self.skip_newlines();
        let headers = self.parse_headers();
        let paragraphs = self.parse_track_body();
        Document { headers, paragraphs, errors: std::mem::take(&mut self.errors) }
    }

    // ── Headers ───────────────────────────────────────────────────

    fn parse_headers(&mut self) -> HeaderSection {
        let mut hs = HeaderSection::default();
        loop {
            match self.peek() {
                Some(Token::KwTitle) | Some(Token::KwSubtitle) | Some(Token::KwComposer)
                | Some(Token::KwTempo) | Some(Token::KwTime) | Some(Token::KwGrouping)
                | Some(Token::KwNote) | Some(Token::KwDivisions) => {
                    self.parse_header_line(&mut hs);
                }
                _ => break,
            }
        }
        hs
    }

    fn parse_header_line(&mut self, hs: &mut HeaderSection) {
        let kw = self.next().unwrap();
        match kw {
            Token::KwTitle => { hs.title = Some(self.parse_header_value()); }
            Token::KwSubtitle => { hs.subtitle = Some(self.parse_header_value()); }
            Token::KwComposer => { hs.composer = Some(self.parse_header_value()); }
            Token::KwTempo => {
                if let Some(Token::Integer(n)) = self.peek() {
                    self.next().ok();
                    hs.tempo = Some(n);
                }
            }
            Token::KwTime => {
                if let (Some(Token::Integer(b)), Some(Token::Slash), Some(Token::Integer(u))) =
                    (self.peek_n(0), self.peek_n(1), self.peek_n(2))
                {
                    self.next().ok(); self.next().ok(); self.next().ok();
                    hs.time = Some((b, u));
                }
            }
            Token::KwGrouping => {
                let mut nums = Vec::new();
                loop {
                    if let Some(Token::Integer(n)) = self.peek() {
                        self.next().ok();
                        nums.push(n);
                    } else { break; }
                    if self.peek() == Some(Token::Plus) { self.next().ok(); }
                    else { break; }
                }
                if !nums.is_empty() { hs.grouping = Some(nums); }
            }
            Token::KwNote => {
                if let (Some(Token::Integer(b)), Some(Token::Slash), Some(Token::Integer(u))) =
                    (self.peek_n(0), self.peek_n(1), self.peek_n(2))
                {
                    self.next().ok(); self.next().ok(); self.next().ok();
                    hs.note = Some((b, u));
                }
            }
            Token::KwDivisions => {
                if let Some(Token::Integer(n)) = self.peek() {
                    self.next().ok();
                    hs.divisions = Some(n);
                }
            }
            _ => unreachable!(),
        }
        self.consume_newline();
    }

    fn parse_header_value(&mut self) -> String {
        let mut parts = Vec::new();
        while let Some(t) = self.peek() {
            if t.is_newline_like() { break; }
            let t = self.next().unwrap();
            parts.push(self.token_text(&t));
        }
        parts.join(" ")
    }

    fn consume_newline(&mut self) {
        if self.peek_raw() == Some(Token::Newline) { self.next_raw().ok(); }
    }

    // ── Track Body ────────────────────────────────────────────────

    fn parse_track_body(&mut self) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();
        let mut current_para = Paragraph::default();
        let mut current_line: Option<TrackLine> = None;

        macro_rules! flush_line {
            () => {
                if let Some(line) = current_line.take() {
                    current_para.lines.push(line);
                }
            };
        }

        macro_rules! commit_para {
            () => {
                flush_line!();
                if !current_para.lines.is_empty() {
                    paragraphs.push(current_para);
                    current_para = Paragraph::default();
                }
            };
        }

        loop {
            match self.peek() {
                None | Some(Token::KwTitle) | Some(Token::KwSubtitle) | Some(Token::KwComposer)
                | Some(Token::KwTempo) | Some(Token::KwTime) | Some(Token::KwGrouping)
                | Some(Token::KwDivisions) => {
                    commit_para!();
                    break;
                }
                Some(Token::KwNote) => {
                    if self.is_paragraph_note_override() {
                        self.next().ok();
                        let b = self.expect_integer().unwrap_or(4);
                        self.expect(Token::Slash).ok();
                        let u = self.expect_integer().unwrap_or(4);
                        commit_para!();
                        current_para = Paragraph { note: Some((b, u)), lines: Vec::new() };
                        self.skip_newlines();
                    } else {
                        self.next().ok();
                        self.push_error(self.last_end, "unexpected 'note' in track body");
                    }
                }
                Some(Token::Newline) => {
                    self.next().ok();
                    if self.peek() == Some(Token::Newline) {
                        self.skip_newlines();
                        commit_para!();
                    }
                }
                Some(_) => {
                    if let Ok(line) = self.parse_track_line() {
                        flush_line!();
                        current_line = Some(line);
                    }
                }
            }
        }
        paragraphs
    }

    fn is_paragraph_note_override(&mut self) -> bool {
        self.peek_n(0);
        matches!(self.peek_n(1), Some(Token::Integer(_)) | Some(Token::Newline))
    }

    fn parse_track_line(&mut self) -> Result<TrackLine, ParseError> {
        let track = self.parse_optional_track_name();
        let mut measures = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Newline) | None => break,
                Some(ref t) if t.is_barline_like() => {
                    if let Some(ms) = self.parse_measure_section()? {
                        measures.push(ms);
                    } else { break; }
                }
                Some(_) => {
                    if let Some(ms) = self.parse_measure_section()? {
                        measures.push(ms);
                    }
                }
            }
        }
        Ok(TrackLine { track, measures })
    }

    fn parse_measure_section(&mut self) -> Result<Option<MeasureSection>, ParseError> {
        let barline = self.parse_barline()?;
        let mut tokens = Vec::new();
        loop {
            match self.peek() {
                None | Some(Token::Newline) => break,
                Some(ref t) if t.is_barline_like() => break,
                Some(_) => { tokens.push(self.parse_measure_expr()?); }
            }
        }
        // Omit trailing empty measures (barline followed by another barline or newline with no content)
        if tokens.is_empty() && matches!(self.peek(), Some(Token::Newline) | None) {
            return Ok(None);
        }
        Ok(Some(MeasureSection { barline, tokens }))
    }

    fn parse_optional_track_name(&mut self) -> Option<String> {
        match self.peek() {
            Some(ref t) if t.is_track_name_glyph() => {
                let t = self.next().unwrap();
                t.glyph_name().map(|s| s.to_string())
            }
            _ => None,
        }
    }

    fn parse_measure_expr(&mut self) -> Result<MeasureExpr, ParseError> {
        match self.peek() {
            Some(Token::MeasureRepeat) => {
                let _t = self.next().unwrap();
                let count = self.lexer.span().len() as u32;
                Ok(MeasureExpr::MeasureRepeat(count))
            }
            Some(Token::MultiRest) => {
                let _t = self.next().unwrap();
                let span = self.lexer.span();
                let s = &self.source[span.start..span.end];
                let count = self.extract_multi_rest_count(s);
                Ok(MeasureExpr::MultiRest(count))
            }
            Some(Token::InlineRepeat) => {
                let _t = self.next().unwrap();
                let span = self.lexer.span();
                let s = &self.source[span.start..span.end];
                let num_str: String = s.chars().skip(1).filter(|c| c.is_ascii_digit()).collect();
                let times: u32 = num_str.parse().unwrap_or(1);
                Ok(MeasureExpr::InlineRepeat(times))
            }
            Some(Token::LBracket) => self.parse_group(),
            Some(Token::LBrace) => {
                let content = self.parse_inline_braced_block()?;
                Ok(MeasureExpr::InlineBracedBlock(content))
            }
            Some(Token::CrescendoStart) => { self.next().ok(); Ok(MeasureExpr::Crescendo) }
            Some(Token::DecrescendoStart) => { self.next().ok(); Ok(MeasureExpr::Decrescendo) }
            Some(Token::HairpinEnd) => { self.next().ok(); Ok(MeasureExpr::HairpinEnd) }
            Some(Token::NavSegno) | Some(Token::NavCoda) => {
                let t = self.next().unwrap();
                let name = self.token_text(&t);
                Ok(MeasureExpr::NavMarker(name))
            }
            Some(Token::NavFine) | Some(Token::NavDC) | Some(Token::NavDS)
            | Some(Token::NavDCalFine) | Some(Token::NavDCalCoda)
            | Some(Token::NavDSalFine) | Some(Token::NavDSalCoda) | Some(Token::NavToCoda) => {
                let t = self.next().unwrap();
                let name = t.nav_name().to_string();
                Ok(MeasureExpr::NavJump(name))
            }
            Some(ref t) if t.is_glyph_like() => self.parse_basic_or_combined(),
            Some(ref t) if t.is_routed_prefix() => {
                let track = t.track_prefix_name().unwrap().to_string();
                self.next().ok();
                let content = self.parse_inline_braced_block()?;
                Ok(MeasureExpr::RoutedBracedBlock { track, content })
            }
            Some(ref t) if t.is_summon_prefix() => {
                let track = t.track_prefix_name().unwrap().to_string();
                self.next().ok();
                let note = self.parse_basic_note()?;
                Ok(MeasureExpr::SummonedNote { track, note })
            }
            Some(_) => {
                let t = self.next().unwrap();
                Err(self.error_at(self.last_end, &format!("unexpected token: {:?}", t)))
            }
            None => Err(self.error_at(self.last_end, "unexpected end of input")),
        }
    }

    fn parse_basic_or_combined(&mut self) -> Result<MeasureExpr, ParseError> {
        let first = self.parse_basic_note()?;
        if self.peek() == Some(Token::Plus) {
            self.next().ok();
            let mut hits = vec![first];
            loop {
                hits.push(self.parse_basic_note()?);
                if self.peek() == Some(Token::Plus) { self.next().ok(); }
                else { break; }
            }
            Ok(MeasureExpr::CombinedHit(hits))
        } else {
            Ok(MeasureExpr::BasicNote(first))
        }
    }

    fn parse_basic_note(&mut self) -> Result<NoteExpr, ParseError> {
        let glyph = match self.next()? {
            ref t if t.is_glyph_like() => t.glyph_name().unwrap().to_string(),
            Token::Rest => "-".to_string(),
            t => return Err(self.error_at(self.last_end, &format!("expected glyph or rest, found {:?}", t))),
        };
        let (dots, halves, stars, modifiers) = self.parse_suffix_chain();
        Ok(NoteExpr { glyph, dots, halves, stars, modifiers })
    }

    fn parse_suffix_chain(&mut self) -> (u32, u32, u32, Vec<String>) {
        let mut dots = 0; let mut halves = 0; let mut stars = 0;
        let mut modifiers = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Dot) => { self.next().ok(); dots += 1; }
                Some(Token::Slash) => { self.next().ok(); halves += 1; }
                Some(Token::Star) => { self.next().ok(); stars += 1; }
                Some(Token::Colon) => {
                    self.next().ok();
                    if let Some(ref t) = self.peek() {
                        if let Some(m) = t.modifier_name() { self.next().ok(); modifiers.push(m.to_string()); }
                        else { break; }
                    } else { break; }
                }
                _ => break,
            }
        }
        (dots, halves, stars, modifiers)
    }

    // ── Group ─────────────────────────────────────────────────────

    fn parse_group(&mut self) -> Result<MeasureExpr, ParseError> {
        self.expect(Token::LBracket)?;
        let n = if let Some(Token::Integer(num)) = self.peek() {
            if self.peek_n(1) == Some(Token::Colon) {
                self.next().ok(); self.next().ok();
                Some(num)
            } else { None }
        } else { None };

        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBracket) => break,
                Some(_) => { items.push(self.parse_measure_expr()?); }
                None => return Err(self.error_at(self.last_end, "unclosed group bracket")),
            }
        }
        self.expect(Token::RBracket)?;

        let mut group_mods = Vec::new();
        while let Some(Token::Colon) = self.peek() {
            self.next().ok();
            if let Some(ref t) = self.peek() {
                if let Some(m) = t.modifier_name() { self.next().ok(); group_mods.push(m.to_string()); }
                else { break; }
            } else { break; }
        }
        Ok(MeasureExpr::Group(GroupExpr { n, items, modifiers: group_mods }))
    }

    // ── Braced Block ──────────────────────────────────────────────

    fn parse_inline_braced_block(&mut self) -> Result<Vec<MeasureExpr>, ParseError> {
        self.expect(Token::LBrace)?;
        let mut content = Vec::new();
        let mut level = 1u32;
        while level > 0 {
            self.skip_trivia();
            match self.peek_raw() {
                Some(Token::LBrace) => {
                    level += 1;
                    self.next_raw().ok();
                    let nested = self.parse_remaining_braced_block()?;
                    content.push(MeasureExpr::InlineBracedBlock(nested));
                }
                Some(Token::RBrace) => {
                    level -= 1;
                    if level > 0 { self.next_raw().ok(); }
                }
                Some(_) => { content.push(self.parse_measure_expr()?); }
                None => return Err(self.error_at(self.last_end, "unclosed brace")),
            }
        }
        self.next_raw().ok();
        Ok(content)
    }

    fn parse_remaining_braced_block(&mut self) -> Result<Vec<MeasureExpr>, ParseError> {
        let mut content = Vec::new();
        let mut level = 1u32;
        while level > 0 {
            self.skip_trivia();
            match self.peek_raw() {
                Some(Token::LBrace) => {
                    level += 1;
                    self.next_raw().ok();
                    let nested = self.parse_remaining_braced_block()?;
                    content.push(MeasureExpr::InlineBracedBlock(nested));
                }
                Some(Token::RBrace) => {
                    level -= 1;
                    if level > 0 { self.next_raw().ok(); }
                }
                Some(_) => { content.push(self.parse_measure_expr()?); }
                None => return Err(self.error_at(self.last_end, "unclosed brace")),
            }
        }
        self.next_raw().ok();
        Ok(content)
    }

    // ── Barline ───────────────────────────────────────────────────

    fn parse_barline(&mut self) -> Result<Barline, ParseError> {
        match self.next()? {
            Token::VoltaRepeatStart => Ok(Barline::VoltaRepeatStart),
            Token::DoubleVoltaTerminator => Ok(Barline::DoubleVoltaTerminator),
            Token::RepeatStart => self.parse_volta_barline("|:"),
            Token::RepeatEnd => self.parse_volta_barline(":|"),
            Token::DoubleBarline => Ok(Barline::Double),
            Token::VoltaTerminator => Ok(Barline::VoltaTerminator),
            Token::Barline => self.parse_volta_barline("|"),
            t => Err(self.error_at(self.last_end, &format!("expected barline, found {:?}", t))),
        }
    }

    fn parse_volta_barline(&mut self, prefix: &str) -> Result<Barline, ParseError> {
        if matches!(self.peek(), Some(Token::Integer(_))) {
            let mut nums = Vec::new();
            loop {
                if let Some(Token::Integer(n)) = self.peek() { self.next().ok(); nums.push(n); }
                else { break; }
                if self.peek() == Some(Token::Comma) { self.next().ok(); }
                else { break; }
            }
            if self.peek() == Some(Token::Dot) {
                self.next().ok();
                Ok(Barline::Volta { prefix: prefix.to_string(), numbers: nums })
            } else {
                Ok(match prefix {
                    "|:" => Barline::RepeatStart,
                    ":|" => Barline::RepeatEnd,
                    _ => Barline::Regular,
                })
            }
        } else {
            Ok(match prefix {
                "|:" => Barline::RepeatStart,
                ":|" => Barline::RepeatEnd,
                "||" => Barline::Double,
                _ => Barline::Regular,
            })
        }
    }
}

// ── Token Extension Methods ──────────────────────────────────────

impl Token {
    fn is_newline_like(&self) -> bool { matches!(self, Token::Newline) }

    fn is_barline_like(&self) -> bool {
        matches!(self,
            Token::Barline | Token::DoubleBarline
            | Token::RepeatStart | Token::RepeatEnd
            | Token::VoltaTerminator | Token::DoubleVoltaTerminator
            | Token::VoltaRepeatStart
        )
    }

    fn is_glyph_like(&self) -> bool { self.glyph_name().is_some() }

    fn is_track_name_glyph(&self) -> bool { self.glyph_name().is_some() }

    fn is_routed_prefix(&self) -> bool {
        self.track_prefix_name().is_some() && matches!(self,
            Token::RouteHH | Token::RouteHF | Token::RouteSD | Token::RouteBD
            | Token::RouteT1 | Token::RouteT2 | Token::RouteT3 | Token::RouteT4
            | Token::RouteRC | Token::RouteC | Token::RouteST
            | Token::RouteBD2 | Token::RouteRC2 | Token::RouteC2
            | Token::RouteSPL | Token::RouteCHN | Token::RouteCB | Token::RouteWB
            | Token::RouteCL
        )
    }

    fn is_summon_prefix(&self) -> bool {
        self.track_prefix_name().is_some() && matches!(self,
            Token::SummonHH | Token::SummonHF | Token::SummonSD | Token::SummonBD
            | Token::SummonT1 | Token::SummonT2 | Token::SummonT3 | Token::SummonT4
            | Token::SummonRC | Token::SummonC | Token::SummonST
            | Token::SummonBD2 | Token::SummonRC2 | Token::SummonC2
            | Token::SummonSPL | Token::SummonCHN | Token::SummonCB | Token::SummonWB
            | Token::SummonCL
        )
    }

    fn nav_name(&self) -> &'static str {
        match self {
            Token::NavFine => "fine", Token::NavDC => "dc", Token::NavDS => "ds",
            Token::NavDCalFine => "dc-al-fine", Token::NavDCalCoda => "dc-al-coda",
            Token::NavDSalFine => "ds-al-fine", Token::NavDSalCoda => "ds-al-coda",
            Token::NavToCoda => "to-coda",
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Document {
        Parser::new(src).parse().expect("parse failed")
    }

    #[test]
    fn test_empty() {
        let doc = parse_ok("");
        assert!(doc.paragraphs.is_empty());
    }

    #[test]
    fn test_header_tempo() {
        let doc = parse_ok("tempo 120\n");
        assert_eq!(doc.headers.tempo, Some(120));
    }

    #[test]
    fn test_header_time() {
        let doc = parse_ok("time 4/4\n");
        assert_eq!(doc.headers.time, Some((4, 4)));
    }

    #[test]
    fn test_simple_track() {
        let doc = parse_ok("HH | x - x - |\n");
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].lines.len(), 1);
        let line = &doc.paragraphs[0].lines[0];
        assert_eq!(line.track.as_deref(), Some("HH"));
        assert_eq!(line.measures.len(), 1);
        assert_eq!(line.measures[0].tokens.len(), 4);
    }

    #[test]
    fn test_track_name_debug() {
        let doc = parse_ok("HH | x |\n");
        let line = &doc.paragraphs[0].lines[0];
        assert_eq!(line.track.as_deref(), Some("HH"));
        assert_eq!(line.measures.len(), 1);
    }

    #[test]
    fn test_combined_hit() {
        let doc = parse_ok("SD | x+d+b |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        match &tokens[0] {
            MeasureExpr::CombinedHit(hits) => assert_eq!(hits.len(), 3),
            _ => panic!("expected CombinedHit"),
        }
    }

    #[test]
    fn test_group() {
        let doc = parse_ok("SD | [x d b] |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        match &tokens[0] {
            MeasureExpr::Group(g) => assert_eq!(g.items.len(), 3),
            _ => panic!("expected Group"),
        }
    }

    #[test]
    fn test_suffix_chain() {
        let doc = parse_ok("SD | x. / * :accent |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        match &tokens[0] {
            MeasureExpr::BasicNote(note) => {
                assert_eq!(note.dots, 1);
                assert_eq!(note.halves, 1);
                assert_eq!(note.stars, 1);
                assert_eq!(note.modifiers, vec!["accent"]);
            }
            _ => panic!("expected BasicNote"),
        }
    }

    #[test]
    fn test_navigation() {
        let doc = parse_ok("HH | @segno x | @dc |\n");
        let m1 = &doc.paragraphs[0].lines[0].measures[0].tokens;
        assert!(matches!(m1[0], MeasureExpr::NavMarker(_)));
        let m2 = &doc.paragraphs[0].lines[0].measures[1].tokens;
        assert!(matches!(m2[0], MeasureExpr::NavJump(_)));
    }

    #[test]
    fn test_measure_repeat() {
        let doc = parse_ok("HH | x | % |\n");
        let measures = &doc.paragraphs[0].lines[0].measures;
        assert_eq!(measures.len(), 2); // | x | and | % | 
    }

    #[test]
    fn test_inline_braced_block() {
        let doc = parse_ok("HH | { x d } |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        match &tokens[0] {
            MeasureExpr::InlineBracedBlock(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected InlineBracedBlock"),
        }
    }

    #[test]
    fn test_nested_braces() {
        let doc = parse_ok("HH | { x { d } b } |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        match &tokens[0] {
            MeasureExpr::InlineBracedBlock(items) => {
                assert_eq!(items.len(), 3);
                match &items[1] {
                    MeasureExpr::InlineBracedBlock(inner) => assert_eq!(inner.len(), 1),
                    _ => panic!("expected nested InlineBracedBlock"),
                }
            }
            _ => panic!("expected InlineBracedBlock"),
        }
    }
}
