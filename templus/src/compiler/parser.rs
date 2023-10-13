use crate::compiler::tokens::Token;

use super::{error::TemplusError, lexer::Lexer};

#[derive(Debug)]
pub enum Expression<'a> {
    Variable(&'a str),
    Literal(&'a str),
    If(IfExpr<'a>, Vec<Statement<'a>>),
    Range(Box<Expression<'a>>, Vec<Statement<'a>>),
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

impl<'a> IfExpr<'a> {
    pub fn eval(&self, ctx: &serde_json::Value) -> Result<bool, TemplusError> {
        match &self.op {
            Some(op) => {
                todo!()
            }
            None => match *self.left {
                Expression::Variable(var) => match ctx.get(var) {
                    Some(v) => match v {
                        serde_json::Value::Null => Ok(false),
                        serde_json::Value::Bool(_v) => Ok(_v.clone()),
                        serde_json::Value::Number(_v) => Ok(_v.as_i64().ok_or(
                            TemplusError::DeafultError("serde maria, what have you done?"),
                        )? != 0),
                        serde_json::Value::String(_v) => Ok(_v.len() > 0),
                        serde_json::Value::Array(_v) => Ok(_v.len() > 0),
                        serde_json::Value::Object(_v) => Ok(true),
                    },
                    None => Ok(false),
                },
                Expression::Literal(_) => Ok(true),
                _ => {
                    return Err(TemplusError::DeafultError(
                        "wtf are you doing in you if statement",
                    ))
                }
            },
        }
    }
}

impl<'a> std::fmt::Display for Statement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Expression(expr) => write!(f, "({})", expr),
            Statement::Block(name, statements) => {
                write!(f, "(block:{})", name)?;
                for stat in statements {
                    write!(f, "{}", stat)?;
                }
                write!(f, "\n")
            }
            Statement::Define(name, statements) => {
                write!(f, "(define:{})", name)?;
                for stat in statements {
                    write!(f, "{}", stat)?;
                }
                write!(f, "\n")
            }
            Statement::Extend(name, statements) => {
                write!(f, "(extends:{})", name)?;
                for stat in statements {
                    write!(f, "{}", stat)?;
                }
                write!(f, "\n")
            }
            Statement::Import(name) => write!(f, "(import:{})", name),
        }
    }
}

impl<'a> std::fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Variable(_) => write!(f, "[var]"),
            Expression::Literal(_) => write!(f, "[lit]"),
            Expression::If(_, _) => write!(f, "[if]"),
            Expression::Range(_, _) => write!(f, "[range]"),
        }
    }
}

impl<'a> std::fmt::Display for IfExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?} {:?}\n", self.left, self.op, self.right)
    }
}

// ---------------------------------------------
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            lexer: Lexer::new(code),
        }
    }

    /// big juicy recursive func
    pub fn parse(&mut self) -> Result<Vec<Statement<'a>>, TemplusError> {
        let mut out = vec![];
        while let Some(token_result) = self.lexer.next() {
            let (token, span) = token_result?;
            match token {
                Token::CodeStart => (), // don't care
                Token::CodeEnd => (),   // don't care
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
                Token::Range => {
                    let var = match self.lexer.next() {
                        Some(Ok((Token::Var(var), _))) => Expression::Variable(var),
                        Some(Ok((Token::Literal(lit), _))) => Expression::Literal(lit),
                        _ => return Err(TemplusError::SyntaxError(("expected var", span))),
                    };
                    let statement =
                        Statement::Expression(Expression::Range(Box::new(var), self.parse()?));
                    out.push(statement);
                }
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
                Token::End => return Ok(out),
                Token::Else => todo!(),
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
