# Templus

### Go inspired template engine/compiler for Rust.

With proper inheritance, composition and caching.

---

Recently, I had the opportunity to develop a small web app in Go utilizing solely the standard libraries.
It is truly remarkable how comprehensive the Go standard library ecosystem is and how far you get with no dependencies at all.

The template behavior was the only truly frustrating aspect for me.
The syntax is acceptable, but the inability to parse all my templates simultaneously, cache the parse tree, and then render specific blocks while maintaining inheritance and composition is disappointing.
Importing items multiple times results in overwrites, and a template can only be inherited once. Go necessitates parsing all required templates for each page in a separate object or reprocessing them with each request.

To challenge myself I implemented my own Go inspired template engine in rust with `serde` being the only dependency.
Syntax is basically the same, but with proper `if` statements, inheritance, and composition.

This is work in progress and will probably be used and iterated on in my next small web project.

```bash
cargo run --example render
```

## Basic Example

Templates are defined by blocks. You can have as many as you want in one file.
Beware. There is very little cloning, the template container is bound to the lifetime of the template source.

```rust
let tmpl = std::fs::read_to_string("templus/examples/example.html").expect("cannot read file");

let mut envirement = templus::renderer::Envirement::new();
envirement.parse(&tmpl).unwrap();

let ctx = Context {
    name: "lommix".to_string(),
    number: 69,
    bool: true,
};

let html = envirement
    .render("foo", &serde_json::to_value(ctx).unwrap())
    .unwrap();
print!("{}", html);
```

Sample template code:
```html
{{ define 'base' }}
<html>
    <head>
        <meta charset="UTF-8" />
        {{block 'meta'}}
        {{ end }}
    </head>
    <body>
        {{ block 'content' }}
        {{ end }}
        {{ block 'js' }}
            <script src="/test.js"></script>
        {{ end }}
    </body>
</html>
{{ end }}

{{ define 'foo' extends 'base' }}
    {{ block 'meta' }}
        <title>jsx sucks</title>
    {{end}}
    {{ block 'content' }}
        <h1>hello</h1>
        {{import 'foobar'}}
        {{ if .bool }}
            <p>bool is true</p>
        {{end}}
        {{ if .number > 42 }}
            <p>num is bigger than 42</p>
        {{ end }}
        {{ if .number < 420 }}
            <p>num is smaller than 420</p>
        {{ end }}
        {{ if .number == 69 }}
            <p>num is 69</p>
        {{ end }}
    {{ end }}
{{ end }}

{{ define 'foobar'}}
    {{ range 10 }}
        <p> you are {{.name}} </p>
    {{ end }}
{{ end }}
```

## Todos

- Variable assignments.
- Binary operators in if statements.
- else
- User defined functions.
- Bindings for other languages.
- Cli Tools and Parse Tree serialization.
