#![allow(unused)]

use crate::compiler::parser::{Parser, Statement};

pub struct Envirement<'a> {
    pub templates: std::collections::HashMap<String, Statement<'a>>,
}

impl<'a> Envirement<'a> {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    pub fn parse(&mut self, template: &str) {
        let mut parser = Parser::new(template.as_bytes());
        let statements = parser.parse().unwrap();
    }
}

#[test]
fn test_render() {
    let tmpl = "
{{ define 'hello' }}
    {{ block 'test'}}
        <h1>hello</h1>
    {{ end }}
    {{ import 'im'}}
{{ end }}

{{define 'lol' extends 'hello'}}
{{ block 'test' }}
    <h1>world</h1>
{{ end }}
{{ end }}

{{ define 'im'}}
    <h1>import</h1>
{{end}}
";

    let mut parser = Parser::new(tmpl.as_bytes());
    let templates = parser.parse().unwrap();

    for template in templates {
        println!("template: {:?}", template);
    }
}
