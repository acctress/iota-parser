# Core Concepts

## Expressions

Everything in Iota is an `Expression<T>`. Expressions are composable, you build complex parsers by combining simple ones.

An `Expression` is an enum under the hood, but you'll never have to construct them yourself, which is why helpers exist in Iota.

## Combining Expressions

Iota overloads `+` and `|` so your parser code reads like a grammar rule.

`+` creates a sequence, both expressions must match in order:
```rust
let key_value = quoted_string() + lit(":") + quoted_string();
```
This matches:
```
"key":"value"
```

`|` creates an ordered choice, the left will be tried first, if that fails the right is tried.
```rust
let flag = lit("true") | lit("false");
```
This matches:
```
"true" or "false"
```

## Actions

An action transforms the matched values of an expression into a single value, this is where your construct your AST nodes.
```rust
let number = action(
    one_or_more(digit),
    |vals| vals.join("")
);
```
This matches:
```
"123" will return "123" as a String
```

`vals` is a flattened vector of any matched sub-expressions. `values()` can be used to filter out any sentinels from `ignore()`.

## Ignoring Values

Some parts of your grammar may be structural, such as brackets, colons, commas etc. and you wouldn't want them showing up in the `vals` vector. You can use `ignore()` to discard them.
```rust
let parens = ignore(ch('(')) + expr() + ignore(ch(')'));
```

> Always use `values()` in your action when using `ignore()`

```rust
    ...
    |vals| {
        let v = values(vals);
    }
    ...
```

## Recursion

PEG parsers like Iota are recursive descent, a rule can refer to itself. But in Rust, constructing a recursive expression tree eagerly causes infinite recursion, so we have to use `lazy()` to defer the construction until parse time.

```rust
fn expr() -> Expression {
    let parenthesised = action(
        ch('(') + lazy(expr) + ch(')'),
        |vals| vals[1].clone(),
    );
    
    parenthesised | number()
}
```

`lazy(expr)` takes a function pointer, since `expr` is a function with no arguments it can be passed directly without a closure.

## Memoisation
`IotaParser` maintains a memo table keyed on `(express pointer, position)`. If the same expression is tried at the same position twice, the cached result is returned instead of a recomputation. This guarentees that even complex recursive grammars run efficiently and do not re parse the same input.

This is the core of packrat parsing, the technique that gives PEG parsers their performance guarantees.

## IotaParser
`IotaParser` owns the input string and the memory table. Create one per parse:

`parse` returns `Option<(usize, Vec<T>)>`. The position after the match and the matched values, or `None` if the expression failed to match.
