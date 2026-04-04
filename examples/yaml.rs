use iota::{
    Expression, IotaParser, ParseValue, action, alpha, alphanum, between, ch, ignore, lazy, lit,
    one_or_more, optional, values, whitespace, ws, zero_or_more,
};

#[derive(Debug, Clone)]
enum YamlValue {
    Str(String),
    Int(i64),
    Map(Vec<(String, YamlValue)>),
}

#[derive(Debug, Clone)]
enum Node {
    Raw(String),
    Value(YamlValue),
    Pair(String, YamlValue),
    Pairs(Vec<(String, YamlValue)>),
}

impl ParseValue for Node {
    fn from_str(s: String) -> Self {
        Node::Raw(s)
    }

    fn as_str(&self) -> String {
        match self {
            Node::Raw(s) => s.clone(),
            _ => String::new(),
        }
    }
}

fn nodes(vals: Vec<Node>) -> Vec<Node> {
    values(vals)
        .into_iter()
        .filter(|v| match v {
            Node::Value(_) | Node::Pair(..) | Node::Pairs(_) => true,
            _ => !v.as_str().is_empty(),
        })
        .collect()
}

fn identifier() -> Expression<Node> {
    action(alpha() + zero_or_more(alphanum()), |vals| {
        Node::Raw(values(vals).iter().map(|v| v.as_str()).collect())
    })
}

fn integer() -> Expression<Node> {
    action(one_or_more(|| iota::digit()), |vals| {
        Node::Raw(values(vals).iter().map(|v| v.as_str()).collect())
    })
}

fn yaml_value() -> Expression<Node> {
    action(integer() | identifier(), |vals| {
        let p = nodes(vals);
        let s = p[0].as_str();

        if let Ok(n) = s.parse::<i64>() {
            Node::Value(YamlValue::Int(n))
        } else {
            Node::Value(YamlValue::Str(s))
        }
    })
}

fn pair() -> Expression<Node> {
    action(
        identifier() + ignore(lit(":")) + ignore(ws()) + yaml_value(),
        |vals| {
            let p = nodes(vals);
            let key = p[0].as_str();
            let val = match &p[1] {
                Node::Value(v) => v.clone(),
                _ => YamlValue::Str(String::new()),
            };

            Node::Pair(key, val)
        },
    )
}

fn entry() -> Expression<Node> {
    nested() | pair()
}

fn map() -> Expression<Node> {
    action(
        entry() + zero_or_more(ignore(lit("\n")) + entry()),
        |vals| {
            let pairs = nodes(vals)
                .into_iter()
                .filter_map(|v| match v {
                    Node::Pair(k, val) => Some((k, val)),
                    _ => None,
                })
                .collect();

            Node::Value(YamlValue::Map(pairs))
        },
    )
}

fn indented_pair() -> Expression<Node> {
    action(ignore(lit("  ")) + pair(), |vals| {
        let p = nodes(vals);
        p.into_iter().find(|v| matches!(v, Node::Pair(..))).unwrap()
    })
}

fn nested() -> Expression<Node> {
    action(
        identifier()
            + ignore(lit(":"))
            + ignore(lit("\n"))
            + indented_pair()
            + zero_or_more(ignore(lit("\n")) + indented_pair()),
        |vals| {
            let p = nodes(vals);
            let key = p[0].as_str();
            let pairs = p[1..]
                .iter()
                .filter_map(|v| match v {
                    Node::Pair(k, val) => Some((k.clone(), val.clone())),
                    _ => None,
                })
                .collect();
            Node::Pair(key, YamlValue::Map(pairs))
        },
    )
}

fn main() {
    let input = "name: John\nage: 30\naddress:\n  city: London\n  zip: SW1A";
    let mut parser = IotaParser::<Node>::new(input.to_string());
    match parser.parse(&map(), 0) {
        Some((_, vals)) => println!("{:#?}", vals[0]),
        None => println!("Parse failed"),
    }
}
