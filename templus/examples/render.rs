#[derive(serde::Serialize)]
struct Context {
    name: String,
    number: i64,
    bool: bool,
}

fn main() {
    let tmpl = std::fs::read_to_string("templus/examples/example.html").expect("cannot read file");

    let mut environment = templus::renderer::Environment::new();
    environment.parse(&tmpl).unwrap();

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
}
