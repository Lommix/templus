#![allow(unused)]

use std::collections::HashMap;

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
                Statement::Define(name, _, _) => {
                    self.templates.insert(name.to_string(), template);
                }
                _ => {
                    return Err(TemplusError::DeafultError(
                        "File contains blocks outside of templates".to_owned(),
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
            .ok_or(TemplusError::DeafultError("template not found".to_owned()))?;

        let mut out = String::new();
        Ok(self.render_stmt(template, ctx, None)?)
    }

    fn extract_blocks(stmt: &'a Vec<Statement<'a>>) -> HashMap<String, &'a Vec<Statement<'a>>> {
        let mut block_map = HashMap::new();
        for st in stmt {
            match st {
                Statement::Block(name, content) => {
                    block_map.insert(name.to_string(), content);
                }
                _ => (),
            };
        }
        block_map
    }

    // mhhhhh recursive functions feel good
    fn render_stmt(
        &self,
        stmt: &Statement<'a>,
        ctx: &serde_json::Value,
        overwrites: Option<&HashMap<String, &'a Vec<Statement<'a>>>>,
    ) -> Result<String, TemplusError> {
        let mut out = String::new();
        match stmt {
            Statement::Expression(expr) => {
                out += (self.render_expr(expr, ctx, overwrites)?).as_str()
            }
            Statement::Define(name, extends, stmts) => match extends {
                Some(extends_name) => {
                    let base_tmpl =
                        self.templates
                            .get(*extends_name)
                            .ok_or(TemplusError::DeafultError(format!(
                                "base template '{}' not found",
                                extends_name
                            )))?;
                    let over = Envirement::extract_blocks(stmts);
                    out += self.render_stmt(base_tmpl, ctx, Some(&over))?.as_str();
                }

                None => {
                    for s in stmts {
                        out += self.render_stmt(s, ctx, overwrites)?.as_str();
                    }
                }
            },
            Statement::Block(name, stmts) => match overwrites {
                Some(ow) => {
                    if let Some(block) = ow.get(*name) {
                        for s in *block {
                            out += self.render_stmt(s, ctx, overwrites)?.as_str();
                        }
                    } else {
                        for s in stmts {
                            out += self.render_stmt(s, ctx, overwrites)?.as_str();
                        }
                    }
                }
                None => {
                    for s in stmts {
                        out += self.render_stmt(s, ctx, overwrites)?.as_str();
                    }
                }
            },
            Statement::Import(tmpl_name) => {
                let tmpl = self
                    .templates
                    .get(*tmpl_name)
                    .ok_or(TemplusError::DeafultError(format!(
                        "Cannot import non existing template: {}",
                        tmpl_name
                    )))?;
                out += self.render_stmt(tmpl, ctx, None)?.as_str();
            }
        }

        Ok(out)
    }

    fn render_expr(
        &self,
        expr: &Expression<'a>,
        ctx: &serde_json::Value,
        overwrites: Option<&HashMap<String, &'a Vec<Statement<'a>>>>,
    ) -> Result<String, TemplusError> {
        let mut out = String::new();

        match expr {
            Expression::Variable(var_name) => {
                out += ctx
                    .get(var_name)
                    .ok_or(TemplusError::DeafultError("var not found".to_owned()))?
                    .as_str()
                    .ok_or(TemplusError::DeafultError("var not stringable".to_owned()))?
            }
            Expression::Literal(literal) => out += literal,
            Expression::If(ifexpr, stmts) => {
                if ifexpr.eval(ctx)? {
                    for s in stmts {
                        out += self.render_stmt(s, ctx, None)?.as_str();
                    }
                }
            }
            Expression::Range(expr, stmts) => match **expr {
                Expression::Variable(var) => {
                    match ctx
                        .get(var)
                        .ok_or(TemplusError::DeafultError("var not found".to_owned()))?
                    {
                        serde_json::Value::Array(array) => {
                            for item in array {
                                for stmt in stmts {
                                    out += self.render_stmt(stmt, item, overwrites)?.as_str();
                                }
                            }
                        }
                        _ => {
                            return Err(TemplusError::DeafultError(
                                "cannot range over non array".to_owned(),
                            ))
                        }
                    }
                }
                Expression::Literal(lit) => {
                    let num = lit.parse::<i64>().map_err(|_| {
                        TemplusError::DeafultError(
                            "range literal required a valid number".to_owned(),
                        )
                    })?;

                    for i in 0..num {
                        for stmt in stmts {
                            // fix key pass trough at some point
                            out += self.render_stmt(stmt, ctx, overwrites)?.as_str();
                        }
                    }
                }
                _ => {
                    return Err(TemplusError::DeafultError(
                        "Can only range over vars or numbers".to_owned(),
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
    num : i64,
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
            num : 10,
        };

        let out = env.render("foo", &serde_json::to_value(ctx).unwrap());
        println!("---------------------------- OUTPUT:");
        print!("{}\n", out.unwrap());
        println!("----------------------------");
    }
}
