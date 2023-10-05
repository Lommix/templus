#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    Template(&'a str),
    Literal(&'a str),
    Var(&'a str),

    Define,
    Extend,
    Import,
    Range,
    Block,
    If,
    Else,
    End,
    Set,

    Eq,         // ==
    Neq,        // !=
    Gte,        // >=
    Gt,         // >
    Lte,        // <=
    Lt,         // <
    And,        // &&
    Or,         // ||
    Assign,     // =
    CodeStart, // {{
    CodeEnd,   // }}
}

impl<'a> Token<'a> {
    pub fn try_from_str(str: &'a str) -> Option<Self> {
        match str {
            "define" => Some(Token::Define),
            "extend" => Some(Token::Extend),
            "block" => Some(Token::Block),
            "import" => Some(Token::Import),
            "range" => Some(Token::Range),
            "else" => Some(Token::Else),
            "if" => Some(Token::If),
            "end" => Some(Token::End),
            "set" => Some(Token::Set),
            "=" => Some(Token::Assign),
            "==" => Some(Token::Eq),
            "!=" => Some(Token::Neq),
            ">=" => Some(Token::Gte),
            ">" => Some(Token::Gt),
            "<=" => Some(Token::Lte),
            "<" => Some(Token::Lt),
            "&&" => Some(Token::And),
            "||" => Some(Token::Or),
            _ => None,
        }
    }
}
