use iota::{
    Expression, IotaParser, ParseValue, action, between, ch, ignore, lit, quoted_string, values, ws,
};

#[derive(Debug, Clone)]
struct JsonObject {
    key: String,
    value: String,
    raw: String,
}

impl ParseValue for JsonObject {
    fn from_str(s: String) -> Self {
        JsonObject {
            key: String::new(),
            value: String::new(),
            raw: s,
        }
    }
    fn as_str(&self) -> String {
        self.raw.clone()
    }
}

fn key_value() -> Expression<JsonObject> {
    action(
        between(
            ch('{'),
            ch('}'),
            quoted_string() + ignore(lit(":")) + ws() + quoted_string(),
        ),
        |vals| {
            let parts = values(vals);
            JsonObject {
                key: parts[0].as_str(),
                value: parts[1].as_str(),
                raw: String::new(),
            }
        },
    )
}

fn main() {
    let mut parser = IotaParser::<JsonObject>::new(r#"{"name": "blinx"}"#.to_string());
    match parser.parse(&key_value(), 0) {
        Some((_, vals)) => println!("{:?}", vals[0]),
        None => println!("Parse failed"),
    }
}
