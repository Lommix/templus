# Templus [Wip]

### Go inspired template engine/compiler for Rust.

With proper inheritance, composition and caching.

---

Recently, I had the opportunity to develop a small web app in Go utilizing solely the standard libraries.
It is truly remarkable how comprehensive the Go standard library ecosystem is and how far you get with no dependencies at all.

The only part that really annoyed me were the templates. The syntax is fine, but the fact that I cannot parse all my templates at once,
cache the parse tree and then only render specific blocks while having inheritance and composition working sucks!
Importing stuff more than once, results in overwrites. Inheriting a template can only be done once.
Go forces you to parse all required templates for each page in separate object or do it on each request again.

As a challenge and because I always wanted to write a parse tree myself with minimal dependencies, I started this project.
Beware it is still work in progress and some features likes user defined functions are missing.

Syntax is basically go template syntax, but with proper if statements and more verbose block types.


## Loading Templates
you just pass in a template string into an environment struct.
```rust
let tmpl = &std::fs::read_to_string("template_file.html").unwrap();
let mut env = Environment::new();
env.parse(tmpl).unwrap();

let ctx = Ctx {
    admin: true,
    name: "lommix".to_string(),
};

let html = env.render("foo", &serde_json::to_value(ctx).unwrap());
```

## Example templates

Files don't matter. Templates have to be defined inside a block.
You can have as many define blocks in one file as you want

```html
{{ define 'base' }}
<html>
    <head>
    <meta charset="UTF-8" />
    {{block 'meta'}} {{ end }}
    </head>
    <body>
        {{ block 'content' }} {{import 'foobar'}} {{ end }}
        {{ block 'js' }} <script src="/test.js"></script> {{ end }}
    </body>
</html>
{{ end }}
```

In contrast to Go, you can also extend defined blocks and overwrite certain parts.

```html
{{ define 'foo' extends 'base' }}
    {{ block 'meta' }}
    <title>jsx sucks</title>
    {{end}}
    {{ block 'content' }}
        <h1>hello</h1>
        {{ if .admin }}
            <h1>admin is logged in</h1>
        {{ end }}
    {{ end }}
{{ end }}
```

There is also composition.

```html
{{ define 'bar' extends 'base' }}
    {{ block 'meta' }} <title>tsx aswell</title> {{end}}
    {{ block 'content' }}
        {{ import 'greet' }}
    {{end}}
{{ end }}

{{ define 'greet' }}
    <p> Welcome, {{ .user }} </p>
{{ end }}
```

For logic, we currently have a loop with `range` and conditions with `if`

```html
{{ define 'jscopium'}}
    {{ if .copium > 420 }}
        <ul>
        {{ range .users }}
            <li>{{.name}} is believing</li>
        {{ end }}
        </ul>
    {{ end }}
{{ end }}
```

## Todos

- Variable assignments.
- Binary operators in if statements.
- User defined functions.
- Bindings for other languages.
- Better Errors when rendering.
- Cli Tools and Parse Tree Binary serialization.
