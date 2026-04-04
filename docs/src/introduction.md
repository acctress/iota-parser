# Introductions

Iota is a PEG parser combinator library for Rust. You can build parsers by composing small, reusable expressions that map directly to grammar rules. There is no code generation, no macros, and no seperate grammar files.

## Features
- Composable expressions with `+` and `|` operators
- Memoised parsing for guaranteed linear time on most grammars.
- Generioc over any AST type via the `ParseValue` trait
- Helpers for common patterns
- Recursive grammars with `lazy()`
