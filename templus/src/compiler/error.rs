use super::lexer::Span;

#[derive(Debug)]
pub enum TemplusError {
    DeafultError(&'static str),
    SyntaxError((&'static str, Span)),
    ParserError(Span),
    LexerError(Span),
}

impl std::error::Error for TemplusError {}

impl std::fmt::Display for TemplusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplusError::DeafultError(msg) => write!(f, "{}", msg),
            TemplusError::SyntaxError((msg, at)) => write!(f, "{} ,at:{}", msg, at),
            TemplusError::ParserError(at) => write!(f, "Parser error at: {}", at),
            TemplusError::LexerError(at) => write!(f, "Lexer error at: {}", at),
        }
    }
}
