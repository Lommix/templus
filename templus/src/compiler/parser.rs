use crate::compiler::tokens::Token;

use super::{error::TemplusError, lexer::Lexer};

#[derive(Debug)]
pub enum Expression<'a> {
    Variable(&'a str),
    Literal(&'a str),
    If(IfExpr<'a>, Vec<Statement<'a>>),
    Range(&'a str, Box<Expression<'a>>, Vec<Statement<'a>>),
    Define(&'a str, Vec<Statement<'a>>),
    Extend(&'a str, Vec<Statement<'a>>),
    Import(&'a str, Vec<Statement<'a>>),
}

#[derive(Debug)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Block(Vec<Statement<'a>>),
}

#[derive(Debug)]
pub struct IfExpr<'a> {
    left: Box<Expression<'a>>,
    right: Box<Expression<'a>>,
    op: BinOp,
}

#[derive(Debug)]
pub enum BinOp {
    And,
    Or,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    state: u8,
}

// ---------------------------------------------

impl<'a> Parser<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            lexer: Lexer::new(code),
            state: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement<'a>>, TemplusError> {
        let out = vec![];

        while let Some(token_result) = self.lexer.next() {
            let (token, span) = token_result?;
            match token {
                Token::CodeStart => todo!(),
                Token::CodeEnd => todo!(),
                Token::Block => todo!(),
                Token::Template(template) => todo!(),
                Token::Literal(literal) => todo!(),
                Token::Var(var) => todo!(),
                Token::Define => todo!(),
                Token::Extend => todo!(),
                Token::Import => todo!(),
                Token::Range => todo!(),
                Token::If => todo!(),
                Token::Else => todo!(),
                Token::End => todo!(),
                Token::Set => todo!(),
                Token::Eq => todo!(),
                Token::Neq => todo!(),
                Token::Gte => todo!(),
                Token::Gt => todo!(),
                Token::Lte => todo!(),
                Token::Lt => todo!(),
                Token::And => todo!(),
                Token::Or => todo!(),
                Token::Assign => todo!(),
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct PtreeNode {
    children: Vec<PtreeNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let tmpl = "{{ define 'hello' }}hello{{ end }}";
        let mut parser = Parser::new(tmpl.as_bytes());
        let templates = parser.parse();
        println!("templates: {:?}", templates);
    }
}
