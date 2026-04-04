use iota::{
    Expression, IotaParser, ParseValue, action, any, ch, digit, ignore, one_or_more, values, ws,
};

#[derive(Debug, Clone)]
struct SExpr {
    op: String,
    args: Vec<String>,
    raw: String,
}

impl ParseValue for SExpr {
    fn from_str(s: String) -> Self {
        SExpr {
            op: String::new(),
            args: vec![],
            raw: s,
        }
    }

    fn as_str(&self) -> String {
        self.raw.clone()
    }
}

fn sexpr() -> Expression<SExpr> {
    action(
        ignore(ch('(')) + any() + ws() + one_or_more(digit) + ws() + one_or_more(digit),
        |vals| {
            let parts = values(vals);
            println!(
                "vals: {:?}",
                parts.iter().map(|v| v.as_str()).collect::<Vec<_>>()
            );
            SExpr {
                op: parts[0].as_str(),
                args: parts[1..].iter().map(|v| v.as_str()).collect(),
                raw: String::new(),
            }
        },
    )
}

fn main() {
    let mut parser = IotaParser::<SExpr>::new("(+ 1 2)".to_string());
    match parser.parse(&sexpr(), 0) {
        Some((_, vals)) => println!("{:?}", vals[0]),
        None => println!("Parse failed"),
    }
}
