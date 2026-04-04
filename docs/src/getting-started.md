# Getting Started

## Installation
```toml
[dependencies]
iota-parser = "0.1.0"
```

## ParseValue Trait

This trait is extremely important for ensuring your types work with Iota. A parser is generic over a type `T` which implements `ParseValue`.
```rust
pub trait ParseValue: Clone {
    fn from_str(s: String) -> Self;
    fn as_str(&self) -> String;
}
```

For simple string parsing, `String` already implements `ParseValue` natively.

For your own AST types, you need to implement it yourself. `from_str` is called for every raw character or literal match, so it's default needs to be sensible, and the real construction of your type happens inside `action` callbacks.
```rust
#[derive(Debug, Clone)]
struct JsonObject {
    key: String,
    value: String,
    raw: String,
}

impl ParseValue for JsonObject {
    fn from_str(s: String) -> Self {
        JsonObject { key: String::new(), value: String::new(), raw: s }
    }
    
    fn as_str(&self) -> String {
        self.raw.clone()
    }
}
```

# Example Parser
```rust
use iota::{IotaParser, action, one_or_more, digit};

fn number() -> Expression {
    action(
        one_or_more(digit),
        |vals| vals.join(""),
    )
}

fn main() {
    let mut parser = IotaParser::::new("123".to_string());
    match parser.parse(&number(), 0) {
        Some((_, vals)) => println!("{:?}", vals[0]),
        None => println!("Parse failed"),
    }
}
```

## Running Examples

Iota comes with some examples you can run directly so you can get a feel for it.
```
cargo run --example sexpr
cargo run --example json
```

* `sexpr` is a simple Lisp s-expression parser e.g. `(+ 1 2)`
* `json` is a simple key-value parser e.g. `{"name": "blinx"}`
