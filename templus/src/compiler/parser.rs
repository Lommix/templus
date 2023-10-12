use crate::compiler::tokens::Token;

use super::{error::TemplusError, lexer::Lexer};

#[derive(Debug)]
pub enum Expression<'a> {
    Variable(&'a str),
    Literal(&'a str),
    If(IfExpr<'a>, Vec<Statement<'a>>),
    Range(&'a str, Box<Expression<'a>>, Vec<Statement<'a>>),
}

#[derive(Debug)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Block(&'a str, Vec<Statement<'a>>),
    Define(&'a str, Vec<Statement<'a>>),
    Extend(&'a str, Vec<Statement<'a>>),
    Import(&'a str), // vars?
}

#[derive(Debug)]
pub struct IfExpr<'a> {
    left: Box<Expression<'a>>,
    right: Option<Box<Expression<'a>>>,
    op: Option<Op>,
}


#[derive(Debug)]
pub enum BinOp {
    And,
    Or,
}

#[derive(Debug)]
pub enum Op {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_node: Option<&'a mut Statement<'a>>,
}

// ---------------------------------------------

impl<'a> Parser<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            lexer: Lexer::new(code),
            current_node: None,
        }
    }

    /// big juicy recursive func
    pub fn parse(&mut self) -> Result<Vec<Statement<'a>>, TemplusError> {
        let mut out = vec![];
        while let Some(token_result) = self.lexer.next() {
            let (token, span) = token_result?;
            match token {
                Token::CodeStart => (), // don't care
                Token::CodeEnd => (),   // dont't care
                Token::Block => {
                    let name = match self.lexer.next() {
                        Some(Ok((Token::Literal(name), _))) => name,
                        Some(Err(err)) => return Err(err),
                        _ => return Err(TemplusError::ParserError(span)),
                    };
                    let statement = Statement::Block(name, self.parse()?);
                    out.push(statement);
                }
                Token::Template(template) => {
                    out.push(Statement::Expression(Expression::Literal(template)));
                }
                Token::Literal(literal) => {}
                Token::Var(var) => {
                    let statement = Statement::Expression(Expression::Variable(var));
                    out.push(statement);
                }
                Token::Define => {
                    let name = match self.lexer.next() {
                        Some(Ok((Token::Literal(name), _))) => name,
                        Some(Err(err)) => return Err(err),
                        _ => return Err(TemplusError::ParserError(span)),
                    };
                    let statement = Statement::Define(name, self.parse()?);
                    out.push(statement);
                }
                Token::Extends => {
                    let name = match self.lexer.next() {
                        Some(Ok((Token::Literal(name), _))) => name,
                        Some(Err(err)) => return Err(err),
                        _ => return Err(TemplusError::ParserError(span)),
                    };
                    let statement = Statement::Extend(name, self.parse()?);
                    out.push(statement);
                }
                Token::Import => {
                    let name = match self.lexer.next() {
                        Some(Ok((Token::Literal(name), _))) => name,
                        Some(Err(err)) => return Err(err),
                        _ => return Err(TemplusError::ParserError(span)),
                    };
                    let statement = Statement::Import(name);
                    out.push(statement);
                }
                Token::Range => todo!(),
                Token::If => {
                    let left = match self.lexer.next() {
                        Some(Ok((Token::Literal(name), _))) => Expression::Literal(name),
                        Some(Ok((Token::Var(name), at))) => Expression::Variable(name),
                        Some(Err(err)) => return Err(err),
                        _ => return Err(TemplusError::ParserError(span)),
                    };

                    match self.lexer.next() {
                        Some(Ok((Token::CodeEnd, _))) => {
                            out.push(Statement::Expression(Expression::If(
                                IfExpr {
                                    left: Box::new(left),
                                    right: None,
                                    op: None,
                                },
                                self.parse()?,
                            )))
                        }
                        _ => {
                            let op = match self.lexer.next() {
                                Some(Ok((Token::Eq, at))) => Op::Eq,
                                Some(Err(err)) => return Err(err),
                                _ => return Err(TemplusError::ParserError(span)),
                            };

                            let right = match self.lexer.next() {
                                Some(Ok((Token::Literal(name), _))) => Expression::Literal(name),
                                Some(Ok((Token::Var(name), _))) => Expression::Variable(name),
                                Some(Err(err)) => return Err(err),
                                _ => return Err(TemplusError::ParserError(span)),
                            };

                            out.push(Statement::Expression(Expression::If(
                                IfExpr {
                                    left: Box::new(left),
                                    right: Some(Box::new(right)),
                                    op: Some(op),
                                },
                                self.parse()?,
                            )))
                        }
                    }
                }
                Token::True => todo!(),
                Token::False => todo!(),
                Token::Else => todo!(),
                Token::End => return Ok(out),
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
        let tmpl = std::fs::read_to_string("1.html").unwrap();
        let mut parser = Parser::new(tmpl.as_bytes());
        let templates = parser.parse().unwrap();
        for template in templates {
            println!("template: {:?}", template);
        }
    }
}
