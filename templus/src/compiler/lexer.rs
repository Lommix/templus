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
            LexerState::InCode(code_buffer) => {
                // is var?
                if code_buffer.starts_with(".") {
                    let offset = code_buffer.find(' ').unwrap_or(code_buffer.len());
                    let var = &code_buffer[1..offset];
                    self.state = LexerState::InCode(code_buffer[offset..].trim());
                    return Some(Ok((Token::Ident(var), self.loc())));
                }

                // string literal starting with '
                if code_buffer.starts_with("'") {
                    let offset = code_buffer[1..].find("'").unwrap_or(code_buffer.len());
                    let literal = &code_buffer[1..offset + 1];

                    self.state = LexerState::InCode(code_buffer[offset + 2..].trim());
                    return Some(Ok((Token::Literal(&literal), self.loc())));
                }

                // string literal starting with "
                if code_buffer.starts_with('"') {
                    let offset = code_buffer[1..].find('"').unwrap_or(code_buffer.len());
                    let literal = &code_buffer[1..offset + 1];

                    self.state = LexerState::InCode(code_buffer[offset + 2..].trim());
                    return Some(Ok((Token::Literal(&literal), self.loc())));
                }

                // number literals, no floats
                if let Some((num,rest)) = split_after_numeric(code_buffer){
                    self.state = LexerState::InCode(rest.trim());
                    return Some(Ok((Token::Literal(&num), self.loc())));
                }

                match code_buffer.split_once(' ') {
                    // handle expressions
                    // todo
                    Some((s, rest)) => {
                        self.state = LexerState::InCode(rest.trim());
                        let token = match Token::try_from_str(s) {
                            Some(token) => token,
                            None => Token::Ident(s),
                        };
                        Some(Ok((token, self.loc())))
                    }
                    None => {
                        if code_buffer.len() > 0 {
                            let token = match Token::try_from_str(code_buffer) {
                                Some(token) => token,
                                None => Token::Ident(code_buffer),
                            };
                            let span = self.loc();
                            self.state = LexerState::InCode("");
                            return Some(Ok((token, span)));
                        }
                        self.state = LexerState::InHtml;
                        Some(Ok((Token::BlockEnd, self.loc())))
                    }
                }
            }
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

fn is_whitespace_only(code: &[u8]) -> bool {
    code.iter().all(|&x| x == b' ' || x == b'\t' || x == b'\n')
}

fn split_after_numeric(code: &str) -> Option<(&str, &str)> {
    let mut offset = 0;
    //get first char
    if !code.chars().nth(0)?.is_numeric(){
        return None;
    }

    while let Some(c) = code.chars().nth(offset) {
        if !c.is_numeric() {
            break;
        }
        offset += 1;
    }
    Some(code.split_at(offset))
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
                && self[start_offset] != b'\t'
                && self[start_offset] != b'\n'
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
    use super::*;

    #[test]
    fn lex_num_split() {
        let code ="123fsd fdsf";
        let (a,b) = split_after_numeric(code).unwrap();
        assert_eq!(a,"123");
        assert_eq!(b,"fsd fdsf");

        let code = "sdf hello wotld";
        assert!(split_after_numeric(code).is_none());
    }

    #[test]
    fn lex_if() {
        let tmpl = "{{ if .loggedIn && .role == 'admin' }} <p>Hello</p> {{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::BlockStart,
            Token::If,
            Token::Ident("loggedIn"),
            Token::And,
            Token::Ident("role"),
            Token::Eq,
            Token::Literal("admin"),
            Token::BlockEnd,
            Token::Template("<p>Hello</p>"),
            Token::BlockStart,
            Token::End,
            Token::BlockEnd,
        ];
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
        });
    }

    #[test]
    fn lex_define_import_extend() {
        let tmpl = "{{ define 'base' }}{{ import 'test' }}{{ extend 'test'}}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::BlockStart,
            Token::Define,
            Token::Literal("base"),
            Token::BlockEnd,
            Token::BlockStart,
            Token::Import,
            Token::Literal("test"),
            Token::BlockEnd,
            Token::BlockStart,
            Token::Extend,
            Token::Literal("test"),
            Token::BlockEnd,
        ];
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
        });
    }

    #[test]
    fn lex_range() {
        let tmpl = "{{ range .users }}{{ .name }}{{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::BlockStart,
            Token::Range,
            Token::Ident("users"),
            Token::BlockEnd,
            Token::BlockStart,
            Token::Ident("name"),
            Token::BlockEnd,
            Token::BlockStart,
            Token::End,
            Token::BlockEnd,
        ];
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
        })
    }

    #[test]
    fn lex_operator() {
        let tmpl = "{{ if .loggedIn && .role == 'admin' || .id >= 100 }} <p>Hello</p> {{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::BlockStart,
            Token::If,
            Token::Ident("loggedIn"),
            Token::And,
            Token::Ident("role"),
            Token::Eq,
            Token::Literal("admin"),
            Token::Or,
            Token::Ident("id"),
            Token::Gte,
            Token::Literal("100"),
            Token::BlockEnd,
            Token::Template("<p>Hello</p>"),
            Token::BlockStart,
            Token::End,
            Token::BlockEnd,
        ];
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
        })
    }
}
