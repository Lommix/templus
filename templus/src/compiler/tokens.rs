#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Html,
    /// Var Token
    Var,
    /// Static
    Const,
    /// Define Token "define"
    Define,
    /// Exends Token "extend"
    Extend,
    /// Include Token "import"
    Import,
    /// In Token "in"
    In,
    /// Comment Token "set"
    Set,
    /// With Token "with"
    With,
    /// For Token "for"
    For,
    /// If Token "if"
    If,
    /// Else Token "else"
    Else,
    /// Endif Token "endif"
    Endif,
    /// Pipe Token "|"
    Pipe,
    /// End Token "end"
    End,
    /// Block Token "block"
    Block,
    /// Func Token
    Func,
    /// assign "="
    Assign,
    /// not equal "!="
    NotEq,
    /// not "!"
    Not,
    /// equal "=="
    Eq,
    /// greater than ">"
    Gt,
    /// less than "<"
    Lt,
    /// and "and"
    And,
    /// or "or"
    Or,
    // comma ","
    Comma,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}
