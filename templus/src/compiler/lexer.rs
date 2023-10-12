use super::{error::TemplusError, tokens::Token};

#[derive(Debug)]
pub struct Span {
    current_line: usize,
    current_column: usize,
    current_offset: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.current_line, self.current_column)
    }
}

#[derive(Debug, Default)]
enum LexerState {
    #[default]
    InHtml,
    InCode,
}

#[derive(Debug, Default)]
pub(crate) struct Lexer<'a> {
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
                        let code = btrim(&self.code[self.cursor..self.code.len()]);
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
                        let code = btrim(&self.code[self.cursor..self.cursor + start]);
                        let token = Token::Template(std::str::from_utf8(code).unwrap());
                        let span = self.loc();
                        self.cursor += start;
                        return Some(Ok((token, span)));
                    }
                }

                // we are at the start of a code block
                // skip block start
                self.cursor += 2;

                let code = &self.code[self.cursor..];
                let block_start_span = self.loc();

                // self.line_cursor += lines_passed;
                // self.cursor += end + 2;

                self.state = LexerState::InCode;
                Some(Ok((Token::CodeStart, block_start_span)))
            }
            LexerState::InCode => {
                self.skip_whitespace();

                if self.cursor >= self.code.len() {
                    return None;
                }

                if &self.code[self.cursor..self.cursor + 2] == b"}}" {
                    self.state = LexerState::InHtml;
                    self.cursor += 2;
                    return Some(Ok((Token::CodeEnd, self.loc())));
                }

                match &self.code[self.cursor] {
                    // string literal
                    b'"' | b'\'' => {
                        let offset =
                            offset_to_delimiter(&self.code[self.cursor..], self.code[self.cursor])
                                .expect("EOF lol");
                        let literal =
                            std::str::from_utf8(&self.code[self.cursor..self.cursor + offset])
                                .unwrap();

                        println!("literal: {:?}", literal);
                        self.cursor += offset + 1;
                        return Some(Ok((Token::Literal(literal), self.loc())));
                    }
                    // number literal
                    b'0'..=b'9' => {
                        let offset =
                            offset_to_number_end(&self.code[self.cursor..]).expect("EOF lol");
                        let number =
                            std::str::from_utf8(&self.code[self.cursor..self.cursor + offset])
                                .unwrap();
                        self.cursor += offset;
                        return Some(Ok((Token::Literal(number), self.loc())));
                    }
                    // var ident
                    b'.' => {
                        self.cursor += 1;
                        let offset =
                            offset_to_delimiter(&self.code[self.cursor..], b' ').expect("EOF lol");
                        let ident =
                            std::str::from_utf8(&self.code[self.cursor..self.cursor + offset])
                                .unwrap();
                        self.cursor += offset;
                        return Some(Ok((Token::Var(ident), self.loc())));
                    }
                    // ident token
                    _ => {
                        let offset =
                            offset_to_delimiter(&self.code[self.cursor..], b' ').expect("EOF lol");

                        let ident = &self.code[self.cursor..self.cursor + offset];
                        self.cursor += offset;
                        let token = Token::try_from_bslice(ident)
                            .expect(&format!("failed at {:?}", std::str::from_utf8(ident)));
                        return Some(Ok((token, self.loc())));
                    }
                }

                // // is var?
                // if code_buffer.starts_with(".") {
                //     let offset = code_buffer.find(' ').unwrap_or(code_buffer.len());
                //     let var = &code_buffer[1..offset];
                //     self.state = LexerState::InCode(code_buffer[offset..].trim());
                //     return Some(Ok((Token::Var(var), self.loc())));
                // }
                //
                // // string literal starting with '
                // if code_buffer.starts_with("'") {
                //     let offset = code_buffer[1..].find("'").unwrap_or(code_buffer.len());
                //     let literal = &code_buffer[1..offset + 1];
                //
                //     self.state = LexerState::InCode(code_buffer[offset + 2..].trim());
                //     return Some(Ok((Token::Literal(&literal), self.loc())));
                // }
                //
                // // string literal starting with "
                // if code_buffer.starts_with('"') {
                //     let offset = code_buffer[1..].find('"').unwrap_or(code_buffer.len());
                //     let literal = &code_buffer[1..offset + 1];
                //
                //     self.state = LexerState::InCode(code_buffer[offset + 2..].trim());
                //     return Some(Ok((Token::Literal(&literal), self.loc())));
                // }
                //
                // // number literals, no floats
                // if let Some((num, rest)) = split_after_numeric(code_buffer) {
                //     self.state = LexerState::InCode(rest.trim());
                //     return Some(Ok((Token::Literal(&num), self.loc())));
                // }
                //
                // //todo: refactor like top
                // match code_buffer.split_once(' ') {
                //     // handle expressions
                //     // todo
                //     Some((s, rest)) => {
                //         self.state = LexerState::InCode(rest.trim());
                //         let token = match Token::try_from_str(s) {
                //             Some(token) => token,
                //             None => panic!("failed at {}", s),
                //         };
                //         Some(Ok((token, self.loc())))
                //     }
                //     None => {
                //         if code_buffer.len() > 0 {
                //             let token = match Token::try_from_str(code_buffer) {
                //                 Some(token) => token,
                //                 None => Token::Var(code_buffer),
                //             };
                //             let span = self.loc();
                //             self.state = LexerState::InCode("");
                //             return Some(Ok((token, span)));
                //         }
                //         self.state = LexerState::InHtml;
                //         Some(Ok((Token::CodeEnd, self.loc())))
                //     }
                // }
            }
        }
    }
}

fn offset_to_delimiter(code: &[u8], delimiter: u8) -> Option<usize> {
    let mut offset = 0;
    loop {
        if offset >= code.len() {
            return None;
        }
        if Some(&delimiter) == code.get(offset) {
            return Some(offset);
        }
        offset += 1;
    }
}

fn offset_to_number_end(code: &[u8]) -> Option<usize> {
    let mut offset = 0;
    loop {
        if offset >= code.len() {
            return None;
        }
        if let Some(b'0'..=b'9') = code.get(offset) {
            offset += 1;
        } else {
            return Some(offset);
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
    if !code.chars().nth(0)?.is_numeric() {
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

fn btrim(input: &[u8]) -> &[u8] {
    if input.is_empty() {
        return input;
    }
    let mut start_offset = 0;
    let mut end_offset = input.len() - 1;

    loop {
        if start_offset >= input.len() {
            break;
        }
        if input[start_offset] != b' '
            && input[start_offset] != b'\t'
            && input[start_offset] != b'\n'
        {
            break;
        }
        start_offset += 1;
    }

    loop {
        if end_offset < 1 {
            break;
        }
        if input[end_offset] == b' ' || input[end_offset] == b'\t' || input[end_offset] == b'\n' {
            end_offset -= 1;
            continue;
        }
        break;
    }
    &input[start_offset..end_offset + 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_num_split() {
        let code = "123fsd fdsf";
        let (a, b) = split_after_numeric(code).unwrap();
        assert_eq!(a, "123");
        assert_eq!(b, "fsd fdsf");

        let code = "sdf hello wotld";
        assert!(split_after_numeric(code).is_none());
    }

    #[test]
    fn lex_if() {
        let tmpl = "{{ if .loggedIn && .role == 'admin' }} <p>Hello</p> {{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::CodeStart,
            Token::If,
            Token::Var("loggedIn"),
            Token::And,
            Token::Var("role"),
            Token::Eq,
            Token::Literal("admin"),
            Token::CodeEnd,
            Token::Template("<p>Hello</p>"),
            Token::CodeStart,
            Token::End,
            Token::CodeEnd,
        ];
        let count = expected.len();
        let mut real = 0;
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
            real += 1;
        });
        assert_eq!(real, count);
    }

    #[test]
    fn lex_extends() {
        let tmpl = "{{ define 'base' extends 'test'}}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::CodeStart,
            Token::Define,
            Token::Literal("base"),
            Token::Extends,
            Token::Literal("test"),
            Token::CodeEnd,
        ];
        let mut timeout = 0;

        println!("parsing : {}", &tmpl);
        for token in lexer {
            println!("{:?}", token);
            timeout += 1;

            if timeout > 100 {
                println!("overflow!!!!");
                break;
            }
        }

        // let data = lexer.collect::<Vec<_>>();
        //
        // for token in &data {
        //     println!("{:?}", token);
        // }
        //
        // assert_eq!(data.len(), expected.len());

        // lexer.zip(expected).for_each(|(a, b)| {
        //     let (t, s) = a.unwrap();
        //     println!("{:?}", t);
        // });
    }
    #[test]
    fn lex_define_import_extend() {
        let tmpl = "{{ define 'base' }}{{ import 'test' }}{{ define 'test' extends 'lol'}}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::CodeStart,
            Token::Define,
            Token::Literal("base"),
            Token::CodeEnd,
            Token::CodeStart,
            Token::Import,
            Token::Literal("test"),
            Token::CodeEnd,
            Token::CodeStart,
            Token::Define,
            Token::Literal("test"),
            Token::Extends,
            Token::Literal("lol"),
            Token::CodeEnd,
        ];
        let count = expected.len();
        let mut real = 0;
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
            real += 1;
        });
        assert_eq!(real, count);
    }

    #[test]
    fn lex_range() {
        let tmpl = "{{ range .users }}{{ .name }}{{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::CodeStart,
            Token::Range,
            Token::Var("users"),
            Token::CodeEnd,
            Token::CodeStart,
            Token::Var("name"),
            Token::CodeEnd,
            Token::CodeStart,
            Token::End,
            Token::CodeEnd,
        ];
        let count = expected.len();
        let mut real = 0;
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
            real += 1;
        });
        assert_eq!(real, count);
    }

    #[test]
    fn lex_operator() {
        let tmpl = "{{block 'test'}}{{ if .loggedIn && .role == 'admin' || .id >= 100 }} <p>Hello</p> {{ end }}{{ end }}";
        let lexer = Lexer::new(tmpl.as_bytes());
        let expected = vec![
            Token::CodeStart,
            Token::Block,
            Token::Literal("test"),
            Token::CodeEnd,
            Token::CodeStart,
            Token::If,
            Token::Var("loggedIn"),
            Token::And,
            Token::Var("role"),
            Token::Eq,
            Token::Literal("admin"),
            Token::Or,
            Token::Var("id"),
            Token::Gte,
            Token::Literal("100"),
            Token::CodeEnd,
            Token::Template("<p>Hello</p>"),
            Token::CodeStart,
            Token::End,
            Token::CodeEnd,
        ];

        let count = expected.len();
        let mut real = 0;
        lexer.zip(expected).for_each(|(a, b)| {
            let (t, s) = a.unwrap();
            assert_eq!(t, b);
            real += 1;
        });
        assert_eq!(real, count);
    }
}
