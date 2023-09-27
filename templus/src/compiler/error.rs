#[derive(Debug)]
pub enum TemplusError {
    UnclosedBlock(String),
    InvalidSyntax,
    InvalidToken(String),
    InvalidExpression(String),
}

impl std::fmt::Display for TemplusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TemplusError {
    fn description(&self) -> &str {
        match self {
            TemplusError::UnclosedBlock(at) => "unclosed block at {}",
            TemplusError::InvalidToken(token) => "invalid token {}",
            TemplusError::InvalidExpression(expr) => "invalid expression {}",
            TemplusError::InvalidSyntax => "invalid syntax",
            _ => "unknown error",
        }
    }
}
