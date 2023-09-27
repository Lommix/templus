#[derive(Debug)]
pub enum TemplusCompilerError {
    UnclosedBlock(String),
    InvalidSyntax,
    InvalidToken(String),
    InvalidExpression(String),
}

impl std::fmt::Display for TemplusCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TemplusCompilerError {
    fn description(&self) -> &str {
        "Syntax error"
    }
}
