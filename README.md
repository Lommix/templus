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

Templates are defined by blocks. You can have as many as you want in one file. Html outside define-blocks is ignored.
There is very little cloning, the template container is bound to the lifetime of the template source. Only in the final
render step, the Html gets copied to new Memory. This should make things pretty fast.

```rust
let tmpl = std::fs::read_to_string("templus/examples/example.html").expect("cannot read file");

let mut envirement = templus::renderer::Envirement::new();
envirement.parse(&tmpl).unwrap();

let html = environment
    .render(
        "foo",
        &templus::context! {
            name => "lommix",
            number => 69,
            bool => true
        },
    )
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

        <div class="import-this">
            {{ import 'foobar' }}
        </div>

        {{ if .bool }}
            <p>bool is true</p>
        {{end}}

        {{ if .number > 42 }}
            <p>num is bigger than 42</p>
        {{ end }}

        {{ if .number < 42 }}
            <p>num is smaller than 42</p>
        {{ else }}
            <p>num is not smaller than 42</p>
        {{ end }}

        {{ if .number == 69 }}
            <p>number is 69</p>
        {{ end }}

    {{ end }}
{{ end }}

{{ define 'foobar'}}
    <ul class="loop-example">
    {{ range 10 }}
        <li> you are {{.name}} </li>
    {{ end }}
    </ul>
{{ end }}
```

## Todos

- Variable assignments.
- Binary operators in if statements.
- User defined functions.
- Bindings for other languages.
- Cli Tools and Parse Tree serialization.
