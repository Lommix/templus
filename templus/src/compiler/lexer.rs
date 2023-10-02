use super::{error::TemplusError, tokens::Token};

#[derive(Debug)]
pub struct Span {
    current_line: usize,
    current_column: usize,
    current_offset: usize,
}

#[derive(Debug, Default)]
enum LexerState<'a> {
    #[default]
    InHtml,
    InCode(&'a str),
}

#[derive(Debug, Default)]
pub(crate) struct Lexer<'a> {
    code: &'a [u8],
    cursor: usize,
    line_cursor: usize,
    state: LexerState<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }

    pub fn skip_whitespace(&mut self) {
        while self.cursor < self.code.len()
            && (self.code[self.cursor].is_ascii_whitespace()
                || self.code[self.cursor].is_ascii_control())
        {
            self.line_cursor += if self.code[self.cursor] == b'\n' {
                1
            } else {
                0
            };
            self.cursor += 1;
        }
    }

    fn loc(&self) -> Span {
        Span {
            current_line: self.line_cursor,
            current_column: self.cursor - self.line_cursor,
            current_offset: self.cursor,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Token<'a>, Span), TemplusError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.code.len() {
            return None;
        }
        match self.state {
            LexerState::InHtml => {
                // find the next punctuation
                let (start, lines_passed) = match next_block_start(&self.code[self.cursor..]) {
                    Some((offset, lines)) => (offset, lines),
                    None => {
                        let code = (&self.code[self.cursor..self.code.len()]).trim();
                        let token = Token::Template(std::str::from_utf8(code).unwrap());
                        let span = self.loc();

                        self.cursor = self.code.len();
                        return Some(Ok((token, span)));
                    }
                };

                self.line_cursor += lines_passed;

                // we found html
                // is it all whitespace?
                if start > 0 {
                    if is_whitespace_only(&self.code[self.cursor..self.cursor + start]) {
                        self.cursor += start;
                    } else {
                        let code = (&self.code[self.cursor..self.cursor + start]).trim();
                        let token = Token::Template(std::str::from_utf8(code).unwrap());
                        let span = self.loc();
                        self.cursor += start;
                        return Some(Ok((token, span)));
                    }
                }

                // we are at the start of a code block
                // skip block start
                self.cursor += 2;

                // find code block end
                let (end, lines_passed) = match next_block_end(&self.code[self.cursor..]) {
                    Some((offset, lines)) => (offset, lines),
                    None => return Some(Err(TemplusError::UnclosedBlock(self.loc()))),
                };

                let code = &self.code[self.cursor..self.cursor + end];
                let block_start_span = self.loc();

                self.line_cursor += lines_passed;
                self.cursor += end + 2;

                self.state = LexerState::InCode(std::str::from_utf8(code).unwrap().trim());
                Some(Ok((Token::BlockStart, block_start_span)))
            }
            // (Ident(" define 'base'"), Span { current_line: 0, current_column: 26, current_offset: 26 })
            LexerState::InCode(code_buffer) => match code_buffer.find(" ") {
                Some(offset) => {
                    self.state = LexerState::InCode((&code_buffer[offset..]).trim());
                    Some(Ok((
                        Token::Ident(code_buffer),
                        self.loc(),
                    )))
                }
                None => {
                    self.state = LexerState::InHtml;
                    Some(Ok((Token::BlockEnd, self.loc())))
                }
            },
        }
    }
}

fn next_block_start(code: &[u8]) -> Option<(usize, usize)> {
    let mut local_offset = 0;
    let mut lines_passed = 0;
    loop {
        if local_offset >= code.len() {
            return None;
        }

        match code.get(local_offset..local_offset + 2) {
            Some(b"{{") => return Some((local_offset, lines_passed)),
            _ => local_offset += 1,
        }

        if let Some(b'\n') = code.get(local_offset) {
            lines_passed += 1;
        }
    }
}

fn next_block_end(code: &[u8]) -> Option<(usize, usize)> {
    let mut local_offset = 0;
    let mut lines_passed = 0;
    loop {
        if local_offset >= code.len() {
            return None;
        }
        match code.get(local_offset..local_offset + 2) {
            Some(b"}}") => return Some((local_offset, lines_passed)),
            _ => local_offset += 1,
        }

        if let Some(b'\n') = code.get(local_offset) {
            lines_passed += 1;
        }
    }
}

/// returns next ident or block end
fn next_ident(code: &[u8]) -> Option<usize> {
    let mut offset = 0;
    loop {
        if offset >= code.len() {
            return None;
        }
        if let Some(b' ') = code.get(offset) {
            return Some(offset);
        }

        offset += 1;
    }
}

fn is_whitespace_only(code: &[u8]) -> bool {
    code.iter().all(|&x| x == b' ' || x == b'\t' || x == b'\n')
}

/// returns the offset of the first occurrence of needle
fn skip_to(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}

/// returns offset and lines passed to needle in haystack
fn skip_to_with_lines(haystack: &[u8], needle: u8) -> Option<(usize, usize)> {
    let mut lines = 0;
    let offset = haystack.iter().position(|&x| {
        lines += if x == b'\n' { 1 } else { 0 };
        return x == needle;
    })?;

    Some((offset, lines))
}

trait Trim {
    fn trim(&self) -> Self;
}

impl Trim for &[u8] {
    fn trim(&self) -> Self {
        if self.is_empty() {
            return self;
        }
        let mut start_offset = 0;
        let mut end_offset = self.len() - 1;

        loop {
            if start_offset >= self.len() {
                break;
            }
            if self[start_offset] != b' ' && self[start_offset] != b'\t' && self[start_offset] != b'\n'
            {
                break;
            }
            start_offset += 1;
        }

        loop {
            if end_offset < 1 {
                break;
            }
            if self[end_offset] == b' ' || self[end_offset] == b'\t' || self[end_offset] == b'\n' {
                end_offset -= 1;
                continue;
            }
            break;
        }
        &self[start_offset..end_offset + 1]
    }
}


#[test]
fn test_lexing() {
    let tmpl = "<html> {{ define 'base' }} {{ import 'test' }} {{ end }} <h2>{{ .Title }}</h2> {{ if true }} <p>hello</p> {{ end }} </html> {{ define 'test '}} <p>test</p> {{ end }}";

    println!("{}", tmpl);
    println!("-----------------------------------------------------------");

    let mut timeout = 0;
    let lexer = Lexer::new(tmpl.as_bytes());
    for token in lexer {
        println!("{:?}", token.unwrap());
        timeout += 1;
        if timeout > 30 {
            break;
        }
    }
    // lexer.for_each(|token| println!("{:?}", token.unwrap()));
}

#[test]
fn find() {
    let tmpl = "01234{{test}}";
    let (offset, line) = next_block_start(tmpl.as_bytes()).unwrap();
    assert_eq!(offset, 5);
}

#[test]
fn test_token_fail() {}
