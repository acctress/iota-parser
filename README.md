# Iota

A PEG parser combinator library for Rust.

## Example
```rust
fn sexpr() -> Expression<SExpr> {
    action(
        between(ch('('), ch(')'), symbol() + ws() + one_or_more(digit) + ws() + one_or_more(digit)),
        |vals| {
            let parts = values(vals);
            SExpr {
                op: parts[0].as_str(),
                args: parts[1..].iter().map(|v| v.as_str()).collect(),
                raw: String::new(),
            }
        }
    )
}
```

## Documentation

Full documentation and examples at [acctress.github.io/iota-parser](https://acctress.github.io/iota-parser).

## Installation
```toml
[dependencies]
iota-parser = "0.1.0"
```

## License

MIT
