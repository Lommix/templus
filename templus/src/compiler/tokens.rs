#[derive(Debug)]
pub(crate) enum Token<'a> {

    Template(&'a str),
    Literal(&'a str),
    Ident(&'a str),

    Var,        // .
    Eq,         // ==
    Neq,        // !=
    Gte,        // >=
    Gt,         // >
    Lte,        // <=
    Lt,         // <
    And,        // &&
    Or,         // ||
    Assign,     // =
    BlockStart, // {{
    BlockEnd,   // }}
}

#[derive(Debug)]
pub(crate) enum Keywords {
    If,
    Else,
    Endif,
    With,
    In,
    Set,
    For,
    EndFor,
    Define,
    Extend,
    Import,
    Block,
    Assign,
    NotEq,
    Eq,
    Gte,
    Gt,
    Lte,
    Lt,
    And,
    Or,
    Comma,
    Undefined,
}
