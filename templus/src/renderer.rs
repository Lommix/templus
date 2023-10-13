#![allow(unused)]

use serde::{Serialize, Serializer};

use crate::compiler::{
    error::TemplusError,
    parser::{Expression, Parser, Statement},
};

pub struct Envirement<'a> {
    pub templates: std::collections::HashMap<String, Statement<'a>>,
}

impl<'a> Envirement<'a> {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    pub fn parse(&mut self, template: &'a str) -> Result<(), TemplusError> {
        let mut parser = Parser::new(template.as_bytes());
        for template in parser.parse()? {
            match template {
                Statement::Define(name, _) | Statement::Extend(name, _) => {
                    self.templates.insert(name.to_string(), template);
                }
                _ => {
                    return Err(TemplusError::DeafultError(
                        "File contains blocks outside of templates",
                    ))
                }
            }
        }
        Ok(())
    }

    pub fn render(&self, name: &str, ctx: &serde_json::Value) -> Result<String, TemplusError> {
        let template = self
            .templates
            .get(name)
            .ok_or(TemplusError::DeafultError("template not found"))?;

        let mut out = String::new();

        Ok(self.render_stmt(template, ctx)?)
    }

    // mhhhhh recursive functions feel good
    fn render_stmt(
        &self,
        stmt: &Statement<'a>,
        ctx: &serde_json::Value,
    ) -> Result<String, TemplusError> {
        let mut out = String::new();
        match stmt {
            Statement::Expression(expr) => out += (self.render_expr(expr, ctx)?).as_str(),
            Statement::Define(name, stmts) => {
                for s in stmts {
                    out += self.render_stmt(s, ctx)?.as_str();
                }
            }
            Statement::Block(name, stmts) => {
                for s in stmts {
                    out += self.render_stmt(s, ctx)?.as_str();
                }
            }
            Statement::Extend(_, _) => todo!(),
            Statement::Import(_) => todo!(),
        }

        Ok(out)
    }

    fn render_expr(
        &self,
        expr: &Expression<'a>,
        ctx: &serde_json::Value,
    ) -> Result<String, TemplusError> {
        let mut out = String::new();

        match expr {
            Expression::Variable(var_name) => {
                out += ctx
                    .get(var_name)
                    .ok_or(TemplusError::DeafultError("var not found"))?
                    .as_str()
                    .ok_or(TemplusError::DeafultError("var not stringable"))?
            }
            Expression::Literal(literal) => out += literal,
            Expression::If(ifexpr, stmts) => {
                if ifexpr.eval(ctx)? {
                    for s in stmts {
                        out += self.render_stmt(s, ctx)?.as_str();
                    }
                }
            }
            Expression::Range(expr, stmts) => match **expr {
                Expression::Variable(var) => {
                    match ctx
                        .get(var)
                        .ok_or(TemplusError::DeafultError("var not found"))?
                    {
                        serde_json::Value::Array(array) => {
                            for item in array {
                                for stmt in stmts {
                                    out += self.render_stmt(stmt, item)?.as_str();
                                }
                            }
                        }
                        _ => return Err(TemplusError::DeafultError("cannot range over non array")),
                    }
                }
                Expression::Literal(lit) => {
                    let num = lit.parse::<i64>().map_err(|_| {
                        TemplusError::DeafultError("range literal required a valid number")
                    })?;

                    for i in 0..num {
                        for stmt in stmts {
                            // fix key pass trough
                            out += self.render_stmt(stmt, ctx)?.as_str();
                        }
                    }
                }
                _ => {
                    return Err(TemplusError::DeafultError(
                        "Can only range over vars or numbers",
                    ))
                }
            },
        }

        Ok(out)
    }
}

#[derive(Serialize)]
struct Ctx {
    admin: bool,
    name: String,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_render() {
        let tmpl = &std::fs::read_to_string("1.html").unwrap();
        let mut env = Envirement::new();
        env.parse(tmpl).unwrap();

        let ctx = Ctx {
            admin: true,
            name: "lommix".to_string(),
        };

        let out = env.render("base", &serde_json::to_value(ctx).unwrap());

        print!("{}", out.unwrap());
    }
}
