#[allow(unused_variables)]
pub mod compiler;
pub mod renderer;

#[macro_export]
macro_rules! context {
    ($($key:ident => $value:expr),*) => {
        {
            let mut map = serde_json::Map::new();
            $(
                map.insert(stringify!($key).to_string(), serde_json::to_value($value).unwrap());
            )*
            serde_json::Value::Object(map)
        }
    };
}
