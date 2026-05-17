use logos::Logos;
use crate::lexer::Token;
use crate::ast::*;
use std::ops::Range;

pub struct Parser<'a> {
    lexer: logos::Lexer<'a, Token>,
    peek_buf: Vec<(Token, Range<usize>)>,
    errors: Vec<ParseError>,
    source: &'a str,
    last_start: usize,
    last_end: usize,
}

impl<'a> Parser<'a> {
    fn is_supported_note_denominator(value: u32) -> bool {
        matches!(value, 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128)
    }

    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Token::lexer(source),
            peek_buf: Vec::new(),
            errors: Vec::new(),
            source,
            last_start: 0,
            last_end: 0,
        }
    }

    pub fn parse(mut self) -> Result<Document, Vec<ParseError>> {
        let doc = self.parse_document();
        if doc.errors.is_empty() {
            Ok(doc)
        } else {
            Err(doc.errors)
        }
    }

    // ── Token Access ──────────────────────────────────────────────

    fn peek(&mut self) -> Option<Token> {
        self.peek_n(0)
    }

    fn peek_raw(&mut self) -> Option<Token> {
        if !self.peek_buf.is_empty() {
            return Some(self.peek_buf[0].0.clone());
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
            // Check for multi-rest pattern before delegating to lexer
            if let Some(mr) = self.try_scan_multi_rest() {
                self.peek_buf.push(mr);
                continue;
            }
            match self.lexer.next() {
                Some(Ok(Token::Space | Token::Comment)) => continue,
                Some(Ok(t)) => self.peek_buf.push((t, self.lexer.span())),
                Some(Err(())) => {
                    let span = self.lexer.span();
                    let text = &self.source[span.start..span.end];
                    self.peek_buf.push((Token::FreeText(text.to_string()), span));
                }
                None => return None,
            }
        }
        Some(self.peek_buf[n].0.clone())
    }

    fn try_scan_multi_rest(&mut self) -> Option<(Token, Range<usize>)> {
        let pos = self.lexer.span().end;
        let rest = &self.source[pos..];
        let bytes = rest.as_bytes();
        if !rest.starts_with("--") { return None; }
        let mut i = 2;
        while bytes.get(i) == Some(&b'-') { i += 1; }
        while bytes.get(i) == Some(&b' ') || bytes.get(i) == Some(&b'\t') { i += 1; }
        let num_start = i;
        if bytes.get(i) == Some(&b'1') {
            i += 1;
            if !bytes.get(i).map_or(false, |b| b.is_ascii_digit()) { return None; }
            while bytes.get(i).map_or(false, |b| b.is_ascii_digit()) { i += 1; }
        } else if bytes.get(i).map_or(false, |b| (b'2'..=b'9').contains(b)) {
            i += 1;
            while bytes.get(i).map_or(false, |b| b.is_ascii_digit()) { i += 1; }
        } else {
            return None;
        }
        let count: u32 = rest[num_start..i].parse().unwrap_or(2);
        while bytes.get(i) == Some(&b' ') || bytes.get(i) == Some(&b'\t') { i += 1; }
        if bytes.get(i) != Some(&b'-') || bytes.get(i + 1) != Some(&b'-') { return None; }
        i += 2;
        while bytes.get(i) == Some(&b'-') { i += 1; }
        for _ in 0..i {
            let _ = self.lexer.next();
        }
        Some((Token::MultiRest(count), pos..pos + i))
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        let (t, span) = if !self.peek_buf.is_empty() {
            self.peek_buf.remove(0)
        } else {
            loop {
                // Check for multi-rest pattern before delegating to lexer
                if let Some(mr) = self.try_scan_multi_rest() {
                    break mr;
                }
                match self.lexer.next() {
                    Some(Ok(Token::Space | Token::Comment)) => continue,
                    Some(Ok(t)) => break (t, self.lexer.span()),
                    Some(Err(())) => {
                        let span = self.lexer.span();
                        let text = &self.source[span.start..span.end];
                        break (Token::FreeText(text.to_string()), span);
                    }
                    None => return Err(self.error_at(self.last_end, "unexpected end of input")),
                }
            }
        };
        self.last_start = span.start;
        self.last_end = span.end;
        Ok(t)
    }

    fn next_raw(&mut self) -> Result<Token, ParseError> {
        if !self.peek_buf.is_empty() {
            self.peek_buf.clear();
        }
        match self.lexer.next() {
            Some(Ok(t)) => {
                let span = self.lexer.span();
                self.last_start = span.start;
                self.last_end = span.end;
                Ok(t)
            }
            Some(Err(())) => {
                let span = self.lexer.span();
                let s = &self.source[span.start..span.end];
                self.last_start = span.start;
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

    fn source_location(&self, offset: usize) -> SourceLocation {
        let (line, column) = self.line_column(offset);
        SourceLocation {
            line,
            column,
            offset: offset as u32,
        }
    }

    fn token_text(&self, t: &Token) -> String {
        match t {
            Token::FreeText(s) => s.clone(),
            // Navigation tokens: return canonical names
            Token::NavSegno => "segno".to_string(),
            Token::NavCoda => "coda".to_string(),
            Token::NavFine => "fine".to_string(),
            Token::NavDC => "dc".to_string(),
            Token::NavDS => "ds".to_string(),
            Token::NavDCalFine => "dc-al-fine".to_string(),
            Token::NavDCalCoda => "dc-al-coda".to_string(),
            Token::NavDSalFine => "ds-al-fine".to_string(),
            Token::NavDSalCoda => "ds-al-coda".to_string(),
            Token::NavToCoda => "to-coda".to_string(),
            // Return actual source character for single-char tokens
            Token::Dot => ".".to_string(),
            Token::Star => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Colon => ":".to_string(),
            Token::Plus => "+".to_string(),
            Token::Comma => ",".to_string(),
            Token::Rest => "-".to_string(),
            Token::CrescendoStart => "<".to_string(),
            Token::DecrescendoStart => ">".to_string(),
            Token::HairpinEnd => "!".to_string(),
            Token::LBrace => "{".to_string(),
            Token::RBrace => "}".to_string(),
            Token::LBracket => "[".to_string(),
            Token::RBracket => "]".to_string(),
            Token::Integer(n) => n.to_string(),
            // Modifier keywords
            Token::ModHalfOpen => "half-open".to_string(),
            Token::ModAccent => "accent".to_string(),
            Token::ModChoke => "choke".to_string(),
            Token::ModClose => "close".to_string(),
            Token::ModCross => "cross".to_string(),
            Token::ModGhost => "ghost".to_string(),
            Token::ModFlam => "flam".to_string(),
            Token::ModDrag => "drag".to_string(),
            Token::ModRoll => "roll".to_string(),
            Token::ModDead => "dead".to_string(),
            Token::ModOpen => "open".to_string(),
            Token::ModBell => "bell".to_string(),
            Token::ModRim => "rim".to_string(),
            // Glyph tokens: return the glyph name
            _ => {
                if let Some(g) = t.glyph_name() {
                    g.to_string()
                } else {
                    format!("{:?}", t)
                }
            }
        }
    }

    fn expect_integer(&mut self) -> Result<u32, ParseError> {
        match self.next()? {
            Token::Integer(n) => Ok(n),
            t => Err(self.error_at(self.last_end, &format!("expected integer, found {:?}", t))),
        }
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
                    if self.line_has_trailing_content() {
                        self.push_error(self.last_end, "invalid tempo header; expected `tempo <int>`");
                        self.consume_line_remainder();
                        return;
                    }
                    hs.tempo = Some(n);
                } else {
                    self.push_error(self.last_end, "invalid tempo header; expected `tempo <int>`");
                    self.consume_line_remainder();
                    return;
                }
            }
            Token::KwTime => {
                if let (Some(Token::Integer(b)), Some(Token::Slash), Some(Token::Integer(u))) =
                    (self.peek_n(0), self.peek_n(1), self.peek_n(2))
                {
                    self.next().ok(); self.next().ok(); self.next().ok();
                    if self.line_has_trailing_content() {
                        self.push_error(self.last_end, "invalid time header; expected `time <int>/<int>`");
                        self.consume_line_remainder();
                        return;
                    }
                    hs.time = Some((b, u));
                } else {
                    self.push_error(self.last_end, "invalid time header; expected `time <int>/<int>`");
                    self.consume_line_remainder();
                    return;
                }
            }
            Token::KwGrouping => {
                let mut nums = Vec::new();
                let mut valid = true;
                let mut expect_num = true;
                loop {
                    if expect_num {
                        if let Some(Token::Integer(n)) = self.peek() {
                            self.next().ok();
                            nums.push(n);
                            expect_num = false;
                        } else {
                            valid = false;
                            break;
                        }
                    } else if self.peek() == Some(Token::Plus) {
                        self.next().ok();
                        expect_num = true;
                    } else {
                        break;
                    }
                }
                if expect_num {
                    valid = false;
                }
                if !valid || nums.is_empty() || self.line_has_trailing_content() {
                    self.push_error(self.last_end, "invalid grouping header; expected `grouping <int>+<int>...`");
                    self.consume_line_remainder();
                    return;
                }
                hs.grouping = Some(nums);
            }
            Token::KwNote => {
                if let (Some(Token::Integer(b)), Some(Token::Slash), Some(Token::Integer(u))) =
                    (self.peek_n(0), self.peek_n(1), self.peek_n(2))
                {
                    self.next().ok(); self.next().ok(); self.next().ok();
                    if b != 1 || !Self::is_supported_note_denominator(u) {
                        self.push_error(self.last_end, "invalid note header; expected `note 1/<power of 2>`");
                        self.consume_line_remainder();
                        return;
                    }
                    if self.line_has_trailing_content() {
                        self.push_error(self.last_end, "invalid note header; expected `note <int>/<int>`");
                        self.consume_line_remainder();
                        return;
                    }
                    hs.note = Some((b, u));
                } else {
                    self.push_error(self.last_end, "invalid note header; expected `note <int>/<int>`");
                    self.consume_line_remainder();
                    return;
                }
            }
            Token::KwDivisions => {
                if let Some(Token::Integer(n)) = self.peek() {
                    self.next().ok();
                    if self.line_has_trailing_content() {
                        self.push_error(self.last_end, "invalid divisions header; expected `divisions <int>`");
                        self.consume_line_remainder();
                        return;
                    }
                    hs.divisions = Some(n);
                } else {
                    self.push_error(self.last_end, "invalid divisions header; expected `divisions <int>`");
                    self.consume_line_remainder();
                    return;
                }
            }
            _ => unreachable!(),
        }
        self.consume_newline();
    }

    fn parse_header_value(&mut self) -> String {
        let start = self.lexer.span().end;
        loop {
            match self.peek_raw() {
                Some(Token::Newline) | None => break,
                Some(_) => {
                    let _ = self.next_raw();
                }
            }
        }
        let end = self.lexer.span().end;
        if end > start && end <= self.source.len() {
            self.source[start..end].trim().to_string()
        } else {
            String::new()
        }
    }

    fn consume_newline(&mut self) {
        // Check buffer first, then lexer
        if self.peek_buf.first().map(|(token, _)| token) == Some(&Token::Newline) {
            self.peek_buf.remove(0);
            return;
        }
        // Use a clone to peek without advancing the real lexer
        let mut iter = self.lexer.clone();
        if iter.next() == Some(Ok(Token::Newline)) {
            self.lexer.next(); // advance past the newline
        }
    }

    fn consume_line_remainder(&mut self) {
        loop {
            match self.peek_raw() {
                Some(Token::Newline) | None => break,
                Some(_) => {
                    let _ = self.next_raw();
                }
            }
        }
        self.consume_newline();
    }

    fn line_has_trailing_content(&mut self) -> bool {
        self.skip_trivia();
        !matches!(self.peek_raw(), Some(Token::Newline) | None)
    }

    // ── Track Body ────────────────────────────────────────────────

    #[allow(unused_assignments)]
    fn parse_track_body(&mut self) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();
        let mut current_para = Paragraph::default();
        let mut current_line: Option<TrackLine> = None;
        let mut at_paragraph_start = true;

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
                at_paragraph_start = true;
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
                    if at_paragraph_start && self.is_complete_paragraph_note_override() {
                        self.next().ok();
                        let b = self.expect_integer().unwrap_or(4);
                        self.expect(Token::Slash).ok();
                        let u = self.expect_integer().unwrap_or(4);
                        if self.line_has_trailing_content() {
                            self.push_error(
                                self.last_end,
                                "invalid paragraph note override; expected `note <int>/<int>`",
                            );
                            self.consume_line_remainder();
                            continue;
                        }
                        commit_para!();
                        current_para = Paragraph { note: Some((b, u)), lines: Vec::new() };
                        self.skip_newlines();
                    } else if at_paragraph_start {
                        self.push_error(self.last_end, "invalid paragraph note override; expected `note <int>/<int>`");
                        self.consume_line_remainder();
                    } else {
                        self.push_error(self.last_end, "unexpected 'note' in track body; paragraph note overrides must appear at paragraph start");
                        self.consume_line_remainder();
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
                        at_paragraph_start = false;
                    }
                }
            }
        }
        paragraphs
    }

    fn is_complete_paragraph_note_override(&mut self) -> bool {
        matches!(
            (self.peek_n(0), self.peek_n(1), self.peek_n(2), self.peek_n(3)),
            (
                Some(Token::KwNote),
                Some(Token::Integer(_)),
                Some(Token::Slash),
                Some(Token::Integer(_)),
            )
        )
    }

    fn parse_track_line(&mut self) -> Result<TrackLine, ParseError> {
        let track = self.parse_optional_track_name();
        let mut measures: Vec<MeasureSection> = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Newline) | None => break,
                Some(Token::RepeatEnd) => {
                    // :| is always a closing barline, never an opening
                    self.next().ok(); // consume :|
                    let repeat_end_location = self.source_location(self.last_start);
                    if let Some(last) = measures.last_mut() {
                        last.closing_barline = Some(Barline::RepeatEnd);
                        last.closing_barline_location = Some(repeat_end_location.clone());
                    }
                    // After :|, there may be a Dot (.→volta terminator) or volta number
                    match self.peek() {
                        Some(Token::Newline) | None => break,
                        Some(Token::Dot) => {
                            self.next().ok(); // consume .
                            if let Some(last) = measures.last_mut() {
                                // Mark as volta-terminator: store in closing info
                                last.closing_barline = Some(Barline::RepeatEndVoltaTerminator);
                                last.closing_barline_location = Some(repeat_end_location);
                            }
                            match self.peek() {
                                Some(Token::Newline) | None => break,
                                _ => {
                                    if let Some(ms) = self.parse_measure_section()? {
                                        measures.push(ms);
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Some(ms) = self.parse_measure_section()? {
                                measures.push(ms);
                            }
                        }
                    }
                }
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
        let barline_location = self.source_location(self.last_start);
        let mut tokens = Vec::new();
        loop {
            match self.peek() {
                None | Some(Token::Newline) => break,
                Some(ref t) if t.is_barline_like() => break,
                Some(_) => { tokens.push(self.parse_measure_expr()?); }
            }
        }
        // Capture distinct closing barlines after the measure payload.
        let mut closing_barline_location = None;
        let closing_barline = match self.peek() {
            Some(Token::RepeatEnd) => {
                self.next().ok(); // consume :|
                closing_barline_location = Some(self.source_location(self.last_start));
                if let Some(Token::Dot) = self.peek() {
                    self.next().ok();
                    Some(Barline::RepeatEndVoltaTerminator)
                } else {
                    Some(Barline::RepeatEnd)
                }
            }
            Some(Token::DoubleBarline) => {
                self.next().ok();
                closing_barline_location = Some(self.source_location(self.last_start));
                Some(Barline::Double)
            }
            Some(Token::VoltaTerminator) => {
                self.next().ok();
                closing_barline_location = Some(self.source_location(self.last_start));
                Some(Barline::VoltaTerminator)
            }
            Some(Token::DoubleVoltaTerminator) => {
                self.next().ok();
                closing_barline_location = Some(self.source_location(self.last_start));
                Some(Barline::DoubleVoltaTerminator)
            }
            _ => None,
        };
        if tokens.is_empty() && closing_barline.is_none() && matches!(self.peek(), Some(Token::Newline) | None) {
            return Ok(None);
        }
        Ok(Some(MeasureSection { barline, barline_location, tokens, closing_barline, closing_barline_location }))
    }

    fn parse_optional_track_name(&mut self) -> Option<String> {
        let next = self.peek();
        if let Some(ref t) = next {
            if t.is_track_name_glyph() {
                let t = self.next().unwrap();
                return t.glyph_name().map(|s| s.to_string());
            }
        }
        None
    }

    fn parse_measure_expr(&mut self) -> Result<MeasureExpr, ParseError> {
        match self.peek() {
            Some(Token::MeasureRepeat) => {
                let _t = self.next().unwrap();
                let count = self.lexer.span().len() as u32;
                Ok(MeasureExpr::MeasureRepeat(count))
            }
            Some(Token::MultiRest(count)) => {
                let _t = self.next().unwrap();
                Ok(MeasureExpr::MultiRest(count))
            }
            Some(Token::InlineRepeat) => {
                let _t = self.next().unwrap();
                let span = self.lexer.span();
                let s = &self.source[span.start..span.end];
                let times: i32 = s
                    .strip_prefix('*')
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(1);
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
            Some(ref t) if t.is_glyph_like() || t.is_summon_prefix() => self.parse_basic_or_combined(),
            Some(Token::Rest) => self.parse_basic_or_combined(),
            Some(ref t) if t.is_routed_prefix() => {
                let track = t.track_prefix_name().unwrap().to_string();
                self.next().ok();
                let content = self.parse_inline_braced_block()?;
                Ok(MeasureExpr::RoutedBracedBlock { track, content })
            }
            Some(_) => {
                let t = self.next().unwrap();
                Err(self.error_at(self.last_end, &format!("unexpected token: {:?}", t)))
            }
            None => Err(self.error_at(self.last_end, "unexpected end of input")),
        }
    }

    fn parse_basic_or_combined(&mut self) -> Result<MeasureExpr, ParseError> {
        let first = if let Some(ref t) = self.peek() { if t.is_summon_prefix() {
            let track = t.track_prefix_name().unwrap().to_string();
            self.next().ok();
            let note = self.parse_basic_note()?;
            MeasureExpr::SummonedNote { track, note }
        } else {
            let note = self.parse_basic_note()?;
            MeasureExpr::BasicNote(note)
        }} else {
            return Err(self.error_at(self.last_end, "expected glyph or summon prefix"));
        };
        if self.peek() == Some(Token::Plus) {
            self.next().ok();
            let mut hits = vec![first];
            loop {
                hits.push(self.parse_single_hit()?);
                if self.peek() == Some(Token::Plus) { self.next().ok(); }
                else { break; }
            }
            Ok(MeasureExpr::CombinedHit(hits))
        } else {
            Ok(first)
        }
    }

    /// Parse a single note or summoned note within a combined hit.
    fn parse_single_hit(&mut self) -> Result<MeasureExpr, ParseError> {
        if let Some(ref t) = self.peek() {
            if t.is_summon_prefix() {
                let track = t.track_prefix_name().unwrap().to_string();
                self.next().ok();
                let note = self.parse_basic_note()?;
                return Ok(MeasureExpr::SummonedNote { track, note });
            }
        }
        let note = self.parse_basic_note()?;
        Ok(MeasureExpr::BasicNote(note))
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
                    level -= 1; // recursive call consumed matching RBrace
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
                    level -= 1; // recursive call consumed matching RBrace
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
        if !matches!(self.peek(), Some(Token::Integer(_))) {
            if let Some(token) = self.peek() {
                if !token.is_barline_like() {
                    return Ok(Barline::Regular);
                }
            }
        }

        // Handle standalone volta number without | prefix (appears after :|)
        if let Some(Token::Integer(n)) = self.peek() {
            self.next().ok();
            let mut nums = vec![n as u32];
            loop {
                if let Some(Token::Integer(n2)) = self.peek() {
                    self.next().ok();
                    nums.push(n2 as u32);
                } else { break; }
                if self.peek() == Some(Token::Comma) { self.next().ok(); }
                else { break; }
            }
            if self.peek() == Some(Token::Dot) {
                self.next().ok();
                return Ok(Barline::Volta { prefix: String::new(), numbers: nums });
            }
            return Err(self.error_at(self.last_end, "expected barline or volta number (e.g. '1.'), found standalone number"));
        }

        match self.next()? {
            Token::VoltaRepeatStart => Ok(Barline::VoltaRepeatStart),
            Token::DoubleVoltaTerminator => Ok(Barline::DoubleVoltaTerminator),
            Token::RepeatStart => self.parse_volta_barline("|:"),
            Token::RepeatEnd => Ok(Barline::RepeatEnd),
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

    fn is_track_name_glyph(&self) -> bool {
        matches!(
            self.glyph_name(),
            Some(
                "HH" | "HF" | "SD" | "BD" | "T1" | "T2" | "T3" | "T4" | "RC" | "C" | "ST"
                    | "BD2" | "RC2" | "C2" | "SPL" | "CHN" | "CB" | "WB" | "CL"
            )
        )
    }

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

    fn parse_err(src: &str) -> Vec<ParseError> {
        Parser::new(src).parse().expect_err("parse should fail")
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
        let line = &doc.paragraphs[0].lines[0];
        assert_eq!(line.track.as_deref(), Some("HH"));
        assert_eq!(line.measures.len(), 1);
        assert_eq!(line.measures[0].tokens.len(), 4);
    }

    #[test]
    fn test_multi_track() {
        let src = "time 4/4\nnote 1/8\ngrouping 2+2\nHH | x - x - |\nSD | --d- --d- |\n";
        let doc = parse_ok(src);
        assert_eq!(doc.paragraphs.len(), 1);
        let lines = &doc.paragraphs[0].lines;
        assert_eq!(lines.len(), 2, "should have HH and SD lines");
        assert_eq!(lines[0].track.as_deref(), Some("HH"));
        assert_eq!(lines[1].track.as_deref(), Some("SD"));
    }

    #[test]
    fn test_dashes_as_rests() {
        // --d- should be parsed as Rest+Rest+d+Rest, not as MultiRest
        let src = "time 4/4\nnote 1/8\ngrouping 2+2\nSD | --d- --d- |\n";
        let doc = parse_ok(src);
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        // Should have 8 tokens: Rest,Rest,Glyphd,Rest,Rest,Rest,Glyphd,Rest
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn test_multi_rest_parser() {
        let src = "time 4/4\nnote 1/8\ngrouping 2+2\nHH | x | --2-- |\n";
        let doc = parse_ok(src);
        let measures = &doc.paragraphs[0].lines[0].measures;
        eprintln!("measures.len={}", measures.len());
        for (i, m) in measures.iter().enumerate() {
            eprintln!("  m[{}]: {} tokens", i, m.tokens.len());
        }
        assert_eq!(measures.len(), 2, "expected 2 measures (x and --2--)");
        let m2_tokens = &measures[1].tokens;
        eprintln!("m2 tokens: {:?}", m2_tokens);
        assert_eq!(m2_tokens.len(), 1, "expected 1 multi-rest token");
        assert!(matches!(m2_tokens[0], MeasureExpr::MultiRest(2)));
    }

    #[test]
    fn test_closing_double_barline_is_preserved() {
        let doc = parse_ok("time 4/4\nnote 1/8\ngrouping 2+2\nSD | d ||\n");
        let measures = &doc.paragraphs[0].lines[0].measures;
        assert_eq!(measures.len(), 1);
        assert!(matches!(measures[0].barline, Barline::Regular));
        assert!(matches!(measures[0].closing_barline, Some(Barline::Double)));
    }

    #[test]
    fn test_repeat_end_volta_terminator_is_distinct_from_double_volta_terminator() {
        let doc = parse_ok("time 4/4\nnote 1/8\ngrouping 2+2\nSD | d :|.\n");
        let measures = &doc.paragraphs[0].lines[0].measures;
        assert_eq!(measures.len(), 1);
        assert!(matches!(measures[0].closing_barline, Some(Barline::RepeatEndVoltaTerminator)));
    }

    #[test]
    fn test_closing_repeat_end_location_is_preserved() {
        let doc = parse_ok("time 4/4\nnote 1/8\ngrouping 2+2\n|: ssss |1. ssSs :|2. cCcc :|\n");
        let measures = &doc.paragraphs[0].lines[0].measures;
        let location = measures[2]
            .closing_barline_location
            .as_ref()
            .expect("expected closing repeat-end location");
        assert_eq!(location.line, 4);
        assert_eq!(location.column, 28);
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
    fn test_title_raw_text() {
        // Unquoted simple text
        let doc = parse_ok("title My Score\n");
        assert_eq!(doc.headers.title.as_deref(), Some("My Score"));

        // With quotes — quotes preserved as-is
        let doc = parse_ok("title \"Quoted Title\"\n");
        assert_eq!(doc.headers.title.as_deref(), Some("\"Quoted Title\""));

        // Single word
        let doc = parse_ok("title Hello\n");
        assert_eq!(doc.headers.title.as_deref(), Some("Hello"));

        // With dots, stas, slashes — all raw
        let doc = parse_ok("title v1.0 / beta * test\n");
        assert_eq!(doc.headers.title.as_deref(), Some("v1.0 / beta * test"));

        // With modifier-like words (should NOT be parsed as modifiers)
        let doc = parse_ok("title accent ghost flam\n");
        assert_eq!(doc.headers.title.as_deref(), Some("accent ghost flam"));

        // With glyph-like characters
        let doc = parse_ok("title x d b s c\n");
        assert_eq!(doc.headers.title.as_deref(), Some("x d b s c"));

        // With mixed special chars
        let doc = parse_ok("title Song No. 5 (Remix) [2024]\n");
        assert_eq!(doc.headers.title.as_deref(), Some("Song No. 5 (Remix) [2024]"));

        // Chinese characters
        let doc = parse_ok("title 李白 李荣浩\n");
        assert_eq!(doc.headers.title.as_deref(), Some("李白 李荣浩"));

        // Subtitle also works the same way
        let doc = parse_ok("subtitle feat. Artist / prod. by G\n");
        assert_eq!(doc.headers.subtitle.as_deref(), Some("feat. Artist / prod. by G"));

        // Composer with dots
        let doc = parse_ok("composer G. Mao\n");
        assert_eq!(doc.headers.composer.as_deref(), Some("G. Mao"));
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

    #[test]
    fn test_malformed_headers_are_errors() {
        let errors = parse_err("time 4\nHH | x |\n");
        assert!(errors.iter().any(|e| e.message.contains("invalid time header")));

        let errors = parse_err("tempo fast\nHH | x |\n");
        assert!(errors.iter().any(|e| e.message.contains("invalid tempo header")));

        let errors = parse_err("grouping 3+\nHH | x |\n");
        assert!(errors.iter().any(|e| e.message.contains("invalid grouping header")));
    }

    #[test]
    fn test_paragraph_note_override_position_and_shape() {
        let errors = parse_err("time 4/4\nnote 1/8\n\nnote\nHH | x - x - |\n");
        assert!(errors.iter().any(|e| e.message.contains("invalid paragraph note override")));

        let errors = parse_err("time 4/4\nnote 1/8\nHH | x - x - |\nnote 1/16\nHH | x x x x x x x x |\n");
        assert!(errors.iter().any(|e| e.message.contains("paragraph note overrides must appear at paragraph start")));
    }

    #[test]
    fn test_inline_repeat_preserves_sign() {
        let doc = parse_ok("HH | x - x - *-1 |\n");
        let tokens = &doc.paragraphs[0].lines[0].measures[0].tokens;
        assert!(matches!(tokens.last(), Some(MeasureExpr::InlineRepeat(-1))));
    }
}
