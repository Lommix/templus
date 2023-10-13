#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    Template(&'a str),
    Literal(&'a str),
    Var(&'a str),

    Define,
    Extends,
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
    pub fn try_from_bslice(bslice: &'a [u8]) -> Option<Self> {
        match bslice {
            b"define" => Some(Token::Define),
            b"extends" => Some(Token::Extends),
            b"block" => Some(Token::Block),
            b"import" => Some(Token::Import),
            b"range" => Some(Token::Range),
            b"else" => Some(Token::Else),
            b"if" => Some(Token::If),
            b"end" => Some(Token::End),
            b"set" => Some(Token::Set),
            b"=" => Some(Token::Assign),
            b"==" => Some(Token::Eq),
            b"!=" => Some(Token::Neq),
            b">=" => Some(Token::Gte),
            b">" => Some(Token::Gt),
            b"<=" => Some(Token::Lte),
            b"<" => Some(Token::Lt),
            b"&&" => Some(Token::And),
            b"||" => Some(Token::Or),
            _ => None,
        }
    }
}

