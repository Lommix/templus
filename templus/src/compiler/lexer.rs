use std::iter::Peekable;

use super::{error::TemplusError, tokens::Token};

#[derive(Debug)]
pub struct TokenSpan<'a> {
    token: Token,
    code: &'a [u8],
    line: usize,
}

impl std::fmt::Display for TokenSpan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}::({}) ->`{}`",
            self.token,
            self.line,
            String::from_utf8_lossy(self.code)
        )
    }
}

#[derive(Debug, Default)]
enum LexerState {
    #[default]
    InHtml,
    InCode,
}

#[derive(Debug, Default)]
pub struct Lexer<'a> {
    code: &'a [u8],
    cursor: usize,
    line_cursor: usize,
    state: LexerState,
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

    pub fn peek(&self) -> Option<TokenSpan<'a>> {
        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenSpan<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.code.len() {
            return None;
        }
        match self.state {
            LexerState::InHtml => {
                // we are in html
                // find the next punctuation
                let (start, lines_passed) = match next_punctuation_start(&self.code[self.cursor..]) {
                    Some((offset, lines)) => (offset, lines),
                    None => {
                        let span = TokenSpan {
                            token: Token::Html,
                            code: (&self.code[self.cursor..self.code.len()]).trim(),
                            line: self.line_cursor,
                        };

                        self.cursor = self.code.len();
                        return Some(span);
                    }
                };

                self.line_cursor += lines_passed;

                // we found html
                if start > 0 {
                    let span = TokenSpan {
                        token: Token::Html,
                        code: (&self.code[self.cursor..self.cursor + start]).trim(),
                        line: self.line_cursor,
                    };
                    self.cursor += start;
                    return Some(span);
                }

                let end_cursor = self.cursor + start + 2;
                let span = TokenSpan {
                    token: Token::Punctuation,
                    code: (&self.code[self.cursor..end_cursor]).trim(),
                    line: self.line_cursor,
                };

                self.line_cursor += lines_passed;
                self.cursor = end_cursor;
                self.state = LexerState::InCode;
                Some(span)
            }
            LexerState::InCode => {

                let (end,lines) = next_punctuation_end(&self.code[self.cursor..]).unwrap();

                let span = TokenSpan {
                    token: Token::Expression,
                    code: (&self.code[self.cursor..self.cursor + end]).trim(),
                    line: self.line_cursor,
                };

                self.state = LexerState::InHtml;
                self.cursor += end;
                Some(span)
            }
        }
    }
}

/// finds the next punctuation start, returns char offset and line offset
fn next_punctuation_start(code: &[u8]) -> Option<(usize, usize)> {
    let mut local_offset = 0;
    loop {
        let (idx, lines) = match skip_to_with_lines(&code[local_offset..], b'{') {
            Some((idx, lines)) => (idx, lines),
            None => return None,
        };

        match code.get(idx + local_offset + 1) {
            Some(b'{') => return Some((idx + local_offset, lines)),
            _ => match idx + local_offset >= code.len() {
                true => return None,
                false => local_offset += idx.max(1),
            },
        }
    }
}

/// finds the next punctuation end, returns char offset and line offset
fn next_punctuation_end(code: &[u8]) -> Option<(usize, usize)> {
    let mut local_offset = 0;
    loop {
        let (idx, lines) = match skip_to_with_lines(&code[local_offset..], b'}') {
            Some((idx, lines)) => (idx, lines),
            None => return None,
        };

        match code.get(idx + local_offset - 1) {
            Some(b'}') => return Some((idx + local_offset, lines)),
            _ => match idx + local_offset >= code.len() {
                true => return None,
                false => local_offset += idx.max(1),
            },
        }
    }
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
            if self[start_offset] != b' '
                || self[start_offset] != b'\t'
                || self[start_offset] != b'\n'
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

#[cfg(test)]
mod tests {
    use super::{next_punctuation_start, Lexer};

    #[test]
    fn test_lexing() {
        let tmpl = "<html> {{ define 'base' }} {{ import 'test' }} {{ end }} <h2>{{ .Title }}</h2> {{ if true }} <p>hello</p> {{ end }} </html> {{ define 'test '}} <p>test</p> {{ end }}";

        let lexer = Lexer::new(tmpl.as_bytes());
        lexer.for_each(|token| println!("{}", token));
    }

    #[test]
    fn find() {
        let tmpl = "01234{{test}}";
        let (offset, line) = next_punctuation_start(tmpl.as_bytes()).unwrap();
        assert_eq!(offset, 5);
    }

    #[test]
    fn test_token_fail() {}
}
