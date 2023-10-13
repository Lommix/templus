use super::{error::TemplusError, lexer::Lexer};
use crate::compiler::tokens::Token;

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
    Define(&'a str, Option<&'a str>, Vec<Statement<'a>>),
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
            Some(op) => match *self.left {
                Expression::Variable(var_name) => {
                    let left_value = ctx
                        .get(var_name)
                        .ok_or(TemplusError::DeafultError("unknown var in if".to_owned()))?;

                    let right_expr = self
                        .right
                        .as_ref()
                        .ok_or(TemplusError::DeafultError("missing right".to_string()))?;

                    let rigth_value = match **right_expr {
                        Expression::Variable(name) => ctx
                            .get(name)
                            .ok_or(TemplusError::DeafultError("".to_string()))?
                            .clone(),
                        Expression::Literal(lit) => {
                            let mut result = serde_json::Value::String(lit.to_string());
                            // is number
                            if let Ok(num) = lit.parse::<i64>() {
                                result = serde_json::Value::Number(serde_json::Number::from(num))
                            }
                            // is bool
                            if lit == "true" || lit == "false" {
                                result = serde_json::Value::Bool(lit.parse::<bool>().unwrap())
                            }
                            result
                        }
                        _ => return Err(TemplusError::DeafultError("".to_string())),
                    };

                    match left_value {
                        serde_json::Value::Null => {
                            Err(TemplusError::DeafultError("null".to_string()))
                        }
                        serde_json::Value::Bool(bool) => IfExpr::eval_bool(bool, op, &rigth_value),
                        serde_json::Value::Number(num) => {
                            IfExpr::eval_number(num.as_i64().unwrap(), op, &rigth_value)
                        }
                        serde_json::Value::String(string) => {
                            IfExpr::eval_string(string, op, &rigth_value)
                        }
                        serde_json::Value::Array(_) => Ok(false),
                        serde_json::Value::Object(_) => Ok(false),
                    }
                }
                _ => {
                    return Err(TemplusError::DeafultError(
                        "if statements require a variable on the left".to_owned(),
                    ))
                }
            },
            None => match *self.left {
                Expression::Variable(var) => match ctx.get(var) {
                    Some(v) => match v {
                        serde_json::Value::Null => Ok(false),
                        serde_json::Value::Bool(_v) => Ok(_v.clone()),
                        serde_json::Value::Number(_v) => {
                            Ok(_v.as_i64().ok_or(TemplusError::DeafultError(
                                "serde maria, what have you done?".to_owned(),
                            ))? != 0)
                        }
                        serde_json::Value::String(_v) => Ok(_v.len() > 0),
                        serde_json::Value::Array(_v) => Ok(_v.len() > 0),
                        serde_json::Value::Object(_v) => Ok(true),
                    },
                    None => Ok(false),
                },
                Expression::Literal(_) => Ok(true),
                _ => {
                    return Err(TemplusError::DeafultError(
                        "wtf are you doing in you if statement".to_owned(),
                    ))
                }
            },
        }
    }

    fn eval_number(left: i64, op: &Op, right: &serde_json::Value) -> Result<bool, TemplusError> {
        println!("{:?}", right.as_i64());
        let num = match right {
            serde_json::Value::Number(num) => num.as_i64().unwrap(),
            _ => {
                return Err(TemplusError::DeafultError(
                    "camparing number to unknown".to_string(),
                ))
            }
        };
        match op {
            Op::Eq => Ok(left == num),
            Op::Neq => Ok(left != num),
            Op::Gt => Ok(left > num),
            Op::Gte => Ok(left >= num),
            Op::Lt => Ok(left < num),
            Op::Lte => Ok(left <= num),
        }
    }

    fn eval_bool(left: &bool, op: &Op, right: &serde_json::Value) -> Result<bool, TemplusError> {
        let bool = match right {
            serde_json::Value::Bool(bool) => bool,
            _ => {
                return Err(TemplusError::DeafultError(
                    "comparing bool with not bool".to_string(),
                ))
            }
        };
        match op {
            Op::Eq => Ok(left == right),
            Op::Neq => Ok(left != right),
            _ => {
                return Err(TemplusError::DeafultError(
                    "bool campare can only be eq or neq".to_string(),
                ))
            }
        }
    }

    fn eval_string(left: &str, op: &Op, right: &serde_json::Value) -> Result<bool, TemplusError> {
        let string = match right {
            serde_json::Value::String(string) => string,
            _ => {
                return Err(TemplusError::DeafultError(
                    "comparing bool with not bool".to_string(),
                ))
            }
        };
        match op {
            Op::Eq => Ok(left == right),
            Op::Neq => Ok(left != right),
            _ => {
                return Err(TemplusError::DeafultError(
                    "string campare can only be eq or neq".to_string(),
                ))
            }
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
            Statement::Define(name, _, statements) => {
                write!(f, "(define:{})", name)?;
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

                    match self.lexer.next() {
                        Some(Ok((Token::Extends, _))) => {
                            match self.lexer.next() {
                                Some(Ok((Token::Literal(extends), _))) => {
                                    let statement =
                                        Statement::Define(name, Some(extends), self.parse()?);
                                    out.push(statement);
                                }
                                _ => {}
                            };
                        }
                        _ => {
                            let statement = Statement::Define(name, None, self.parse()?);
                            out.push(statement);
                        }
                    }
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
                        _ => {
                            return Err(TemplusError::SyntaxError((
                                "expected var".to_owned(),
                                span,
                            )))
                        }
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
                        _op => {
                            let op = match _op {
                                Some(Ok((Token::Eq, at))) => Op::Eq,
                                Some(Ok((Token::Neq, at))) => Op::Neq,
                                Some(Ok((Token::Gt, at))) => Op::Gt,
                                Some(Ok((Token::Gte, at))) => Op::Gte,
                                Some(Ok((Token::Lt, at))) => Op::Lt,
                                Some(Ok((Token::Lte, at))) => Op::Lte,
                                Some(Err(err)) => return Err(err),
                                _t => {
                                    return Err(TemplusError::DeafultError(format!(
                                        "error with {:?}",
                                        _t
                                    )))
                                }
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
                _any => {
                    return Err(TemplusError::DeafultError(format!(
                        "this token is supposed to be be here: {:?}",
                        _any
                    )))
                }
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
