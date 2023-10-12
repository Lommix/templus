use super::lexer::Span;

#[derive(Debug)]
pub enum TemplusError {
    UnclosedBlock(Span),
    InvalidSyntax,
    InvalidToken(Span),
    InvalidExpression(Span),
    ParserError((String,Span)),
    LexerError((String,Span)),
}

impl std::error::Error for TemplusError {}


impl std::fmt::Display for TemplusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplusError::UnclosedBlock(at) => write!(f, "Unclosed block at {:?}", at),
            TemplusError::InvalidToken(token) => write!(f, "Invalid token {:?}", token),
            TemplusError::InvalidExpression(expr) => write!(f, "Invalid expression {:?}", expr),
            TemplusError::InvalidSyntax => write!(f, "Invalid syntax"),
            TemplusError::ParserError((msg, at)) => write!(f, "Parser error: {} -- at: {}", msg, at),
            TemplusError::LexerError((msg, at)) => write!(f, "Lexer error: {} -- at: {}", msg, at),
        }
    }
}

