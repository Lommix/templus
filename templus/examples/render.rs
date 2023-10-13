#[derive(serde::Serialize)]
struct Context {
    name: String,
    number: i64,
    bool: bool,
}

fn main() {
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
}
