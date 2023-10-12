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
    pub fn try_from_str(str: &'a str) -> Option<Self> {
        match str {
            "define" => Some(Token::Define),
            "extends" => Some(Token::Extends),
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

