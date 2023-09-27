use std::default;

use super::{error::TemplusError, tokens::Token};

/// representation of a single token
#[derive(Debug)]
pub struct TokenSpan<'a> {
    token: Token,
    code: &'a [u8],
}

impl std::fmt::Display for TokenSpan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}  -> {}",
            self.token,
            String::from_utf8_lossy(self.code)
        )
    }
}

#[derive(Debug)]
enum LexerState {
    InComment,
    InCode,
    InVar,
    InHtml,
}

#[derive(Debug, PartialEq, Eq)]
enum BlockType {
    Code,
    Comment,
    Var,
}

/// The Lexer
/// takes a byte sequence of template code and provides an iterator over all tokens in order
#[derive(Debug)]
pub struct Lexer<'a> {
    code: &'a [u8],
    cursor: usize,
    state: LexerState,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            code,
            cursor: 0,
            state: LexerState::InHtml,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<TokenSpan<'a>, TemplusError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            LexerState::InHtml => {
                let (start_type, start_offset) =
                    match find_next_block_start(&self.code, self.cursor) {
                        Some((start_type, start_offset)) => (start_type, start_offset),
                        None => {
                            self.cursor += skip_whitespace(&self.code, self.cursor).unwrap_or(0);
                            if self.cursor >= self.code.len() {
                                return None;
                            }
                            let span = Some(Ok(TokenSpan {
                                token: Token::Html,
                                code: &self.code[self.cursor..],
                            }));
                            self.cursor = self.code.len();
                            return span;
                        }
                    };

                if start_offset > 0 {
                    let span = TokenSpan {
                        token: Token::Html,
                        code: &self.code[self.cursor..self.cursor + start_offset],
                    };
                    self.cursor += start_offset;
                    return Some(Ok(span));
                }

                self.cursor += 2;
                self.cursor += skip_whitespace(&self.code, self.cursor).unwrap_or(0);

                match start_type {
                    BlockType::Var => self.state = LexerState::InVar,
                    BlockType::Code => self.state = LexerState::InCode,
                    BlockType::Comment => self.state = LexerState::InComment,
                };

                return self.next();
            }
            LexerState::InCode => {
                let (end_type, end_offset) = match find_next_block_end(&self.code, self.cursor) {
                    Some((end_type, end_offset)) => (end_type, end_offset),
                    None => return Some(Err(TemplusError::UnclosedBlock("".to_string()))),
                };

                if end_type != BlockType::Code {
                    return Some(Err(TemplusError::UnclosedBlock("".to_string())));
                }

                let span = TokenSpan {
                    token: Token::Block,
                    code: &self.code[self.cursor..self.cursor + end_offset - 2],
                };

                self.cursor += end_offset + 1;
                self.cursor += skip_whitespace(&self.code, self.cursor).unwrap_or(0);
                self.state = LexerState::InHtml;

                return Some(Ok(span));
            }
            LexerState::InVar => {
                let (end_type, end_offset) = match find_next_block_end(&self.code, self.cursor) {
                    Some((end_type, end_offset)) => (end_type, end_offset),
                    None => return Some(Err(TemplusError::UnclosedBlock("".to_string()))),
                };

                if end_type != BlockType::Var {
                    return Some(Err(TemplusError::InvalidSyntax));
                }

                let span = TokenSpan {
                    token: Token::Var,
                    code: &self.code[self.cursor..self.cursor + end_offset - 2],
                };

                self.cursor += end_offset + 1;
                self.cursor += skip_whitespace(&self.code, self.cursor).unwrap_or(0);
                self.state = LexerState::InHtml;

                return Some(Ok(span));
            }
            LexerState::InComment => {
                let (end_type, end_offset) = match find_next_block_end(&self.code, self.cursor) {
                    Some((end_type, end_offset)) => (end_type, end_offset),
                    None => return Some(Err(TemplusError::UnclosedBlock("".to_string()))),
                };

                if end_type != BlockType::Comment {
                    return Some(Err(TemplusError::InvalidSyntax));
                }

                let span = TokenSpan {
                    token: Token::Comment,
                    code: &self.code[self.cursor..self.cursor + end_offset - 2],
                };

                self.cursor += end_offset + 1;
                self.cursor += skip_whitespace(&self.code, self.cursor).unwrap_or(0);
                self.state = LexerState::InHtml;

                return Some(Ok(span));
            }
        }
    }
}

fn find_next_block_start(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut local_offset = 0;
    loop {
        let idx = match skip_to(&code[(offset + local_offset)..], b'{') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx + local_offset + 1) {
            Some(b'{') => return Some((BlockType::Var, idx + local_offset)),
            Some(b'%') => return Some((BlockType::Code, idx + local_offset)),
            Some(b'#') => return Some((BlockType::Comment, idx + local_offset)),
            _ => match offset + idx + local_offset >= code.len() {
                true => return None,
                false => local_offset += idx.max(1),
            },
        }
    }
}

fn find_next_block_end(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut local_offset = 0;
    loop {
        let idx = match skip_to(&code[(offset + local_offset)..], b'}') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx + local_offset - 1) {
            Some(b'}') => return Some((BlockType::Var, idx + local_offset)),
            Some(b'%') => return Some((BlockType::Code, idx + local_offset)),
            Some(b'#') => return Some((BlockType::Comment, idx + local_offset)),
            _ => match offset + idx + local_offset >= code.len() {
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

fn skip_whitespace(code: &[u8], offset: usize) -> Option<usize> {
    code.iter().position(|&x| x != b' ')
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    #[test]
    fn test_lexing() {
        let tmpl = "<html>{% block 'html' %}<p>Hello</p>{% end %}{% block js %}<script>alert('{{ foo }}')</script>{% end %}</html>";
        let mut lexer = Lexer::new(tmpl.as_bytes());
        lexer.for_each(|x| println!("{}", x.unwrap()));
        // for _ in 0..30 {
        //     match lexer.next() {
        //         Some(Ok(t)) => println!("{}", t),
        //         _ => break,
        //     }
        // }
    }
}
