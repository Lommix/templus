use super::{
    error::TemplusCompilerError,
    tokenizer::{BlockType, Tokenizer},
    tokens::Token,
};

/// representation of a single token
#[derive(Debug)]
pub struct TokenSpan<'a> {
    token: Token,
    code: &'a str,
}

/// The Lexer
/// takes a byte sequence of template code and provides an iterator over all tokens in order
pub struct Lexer<'a> {
    tokenizer: Tokenizer<'a>,
    current_stream: Option<&'a str>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Lexer<'a> {
        Lexer {
            tokenizer: Tokenizer::new(source),
            current_stream: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<TokenSpan<'a>, TemplusCompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.next() {
            Some(token) => match token {
                Ok(block_span) => {
                    // continue current stream?
                    match self.current_stream {
                        Some(stream) => {}
                        None => (),
                    };

                    // next stream
                    match block_span.block_type() {
                        BlockType::Html => {
                            return Some(Ok(TokenSpan {
                                code: block_span.code(),
                                token: Token::Html,
                            }));
                        }
                        BlockType::Code => {
                        }
                        BlockType::Variable => {
                            return Some(Ok(TokenSpan {
                                code: block_span.code(),
                                token: Token::Var,
                            }));
                        }
                        _ => {}
                    }

                    let span = Some(Ok(TokenSpan {
                        code: "",
                        token: Token::In,
                    }));
                    return span;
                }
                Err(err) => Some(Err(err)),
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;

    #[test]
    fn test_lexing() {
        let tmpl = "<html>{% block 'html' %}<p>Hello</p>{% end %}{% block js %}<script>alert('{{ foo }}')</script>{% end %}</html>";
        let lexer = Lexer::new(tmpl.as_bytes());

        lexer.for_each(|token| {
            println!("{:?}", token.unwrap());
        })
    }
}
