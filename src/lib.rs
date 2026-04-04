use std::{
    char,
    collections::HashMap,
    ops::{Add, BitOr},
};

const IGNORED: &str = "\u{E000}";

pub trait ParseValue: Clone {
    fn from_str(s: String) -> Self;
    fn as_str(&self) -> String;
}

impl ParseValue for String {
    fn from_str(s: String) -> Self {
        s
    }
    fn as_str(&self) -> String {
        self.clone()
    }
}

/// Primitve expression types for IotaParser
pub enum Expression<T> {
    Literal(String),
    Char(char),
    CharRange(char, char),
    Sequence(Vec<Expression<T>>),
    OrderedChoice(Box<Expression<T>>, Box<Expression<T>>),
    ZeroOrMore(Box<Expression<T>>),
    Not(Box<Expression<T>>),
    Action(Box<Expression<T>>, Box<dyn Fn(Vec<T>) -> T>),
    Lazy(Box<dyn Fn() -> Expression<T>>),
}

pub struct IotaParser<T> {
    memo_table: HashMap<(usize, usize), Option<(usize, Vec<T>)>>,
    input: String,
}

/// Literal helper, construct a Literal from a &str
pub fn lit<T>(s: &str) -> Expression<T> {
    Expression::Literal(s.to_string())
}

/// Char helper, construct a Char from a char
pub fn ch<T>(c: char) -> Expression<T> {
    Expression::Char(c)
}

/// Digit char helper, constructs a CharRange from '0' to '9'
pub fn digit<T>() -> Expression<T> {
    Expression::CharRange('0', '9')
}

/// Alpha char helper, constructs a CharRange from 'a' to 'z' and 'A' to 'Z'
pub fn alpha<T>() -> Expression<T> {
    Expression::CharRange('a', 'z') | Expression::CharRange('A', 'Z')
}

/// Alpha numeric char helper, constructs a CharRange from 'a' to 'z', 'A' to 'Z', and '0' to '9'
pub fn alphanum<T>() -> Expression<T> {
    Expression::CharRange('a', 'z')
        | Expression::CharRange('A', 'Z')
        | Expression::CharRange('0', '9')
}

/// Whitespace char helper, recognizes whitespaces, tabs and new lines.
pub fn whitespace<T>() -> Expression<T> {
    Expression::Char(' ') | Expression::Char('\t') | Expression::Char('\n')
}

/// Helper for recognising any character from char::MIN to char::MAX
pub fn any<T>() -> Expression<T> {
    Expression::CharRange(char::MIN, char::MAX)
}

/// Helper for matching an expression zero or more times, collection all results.
/// This helper will always succeed and will return an empty list IF the expression never
/// matches.
pub fn zero_or_more<T>(e: Expression<T>) -> Expression<T> {
    Expression::ZeroOrMore(Box::new(e))
}

/// Helper for matching an expression one or more times. Will fail if the expression doesn't
/// match at least once
pub fn one_or_more<T>(f: impl Fn() -> Expression<T>) -> Expression<T> {
    f() + zero_or_more(f())
}

/// Matches an expression zero or one times, this will always succeed.
pub fn optional<T>(f: impl Fn() -> Expression<T>) -> Expression<T> {
    f() | lit("")
}

/// Helper which wraps an expression in an action that transforms it's matched values into
/// a new value.
pub fn action<T>(expr: Expression<T>, f: impl Fn(Vec<T>) -> T + 'static) -> Expression<T> {
    Expression::Action(Box::new(expr), Box::new(f))
}

/// Helper which defers construction of an expression until parse time.
/// This should be used to break recursive grammar cycles.
pub fn lazy<T>(f: impl Fn() -> Expression<T> + 'static) -> Expression<T> {
    Expression::Lazy(Box::new(f))
}

/// Helper for matching common mathematical symbols.
pub fn symbol<T: ParseValue>() -> Expression<T> {
    ch('+') | ch('-') | ch('*') | ch('/')
}

/// Helper for matching quoted strings.
pub fn quoted_string<T: ParseValue>() -> Expression<T> {
    action(
        ch('"')
            + zero_or_more(action(
                Expression::Not(Box::new(ch::<T>('"'))) + any(),
                |vals| T::from_str(vals[1].as_str()),
            ))
            + ch('"'),
        |vals| {
            let inner = vals[1..vals.len() - 1]
                .iter()
                .map(|v| v.as_str())
                .collect::<String>();
            T::from_str(inner)
        },
    )
}

/// Helper for matching quoted strings in a raw format.
pub fn quoted_string_raw() -> Expression<String> {
    action(
        ch::<String>('"')
            + zero_or_more(action(
                Expression::Not(Box::new(ch::<String>('"'))) + any::<String>(),
                |vals| vals[1].clone(),
            ))
            + ch::<String>('"'),
        |vals| vals[1..vals.len() - 1].iter().map(|v| v.as_str()).collect(),
    )
}

/// Helper which matches an expression but discards its value, emitting a sentinel instead.
/// (note: this should be replaced with something else)
/// Using `values` in actions will filter these out.
pub fn ignore<T: ParseValue>(e: Expression<T>) -> Expression<T> {
    action(e, |_| T::from_str(IGNORED.to_string()))
}

/// Filters out sentinel values produced by `ignore` from a values list.
/// Should be called at the start of every action that uses ignored expressions.
pub fn values<T: ParseValue>(vals: Vec<T>) -> Vec<T> {
    vals.into_iter().filter(|v| v.as_str() != IGNORED).collect()
}

/// Helper which ignores zero or more whitespace characters.
pub fn ws<T: ParseValue>() -> Expression<T> {
    ignore(zero_or_more(whitespace()))
}

/// Helper which wraps an inner expression between ignored delimiters.
/// # Example
/// ```
/// let parens = between(ch('('), ch(')'), expr());
/// ```
pub fn between<T: ParseValue>(
    open: Expression<T>,
    close: Expression<T>,
    inner: Expression<T>,
) -> Expression<T> {
    ignore(open) + inner + ignore(close)
}

/// Helper which matches items seperated by a seperator expression,
/// all item values are collected.
pub fn sep_by<T: ParseValue>(
    item: impl Fn() -> Expression<T>,
    separator: impl Fn() -> Expression<T>,
) -> Expression<T> {
    item() + zero_or_more(ignore(separator()) + item())
}

impl<T> IotaParser<T>
where
    T: ParseValue,
{
    pub fn new(input: String) -> Self {
        IotaParser {
            memo_table: HashMap::new(),
            input,
        }
    }

    pub fn parse(&mut self, expr: &Expression<T>, pos: usize) -> Option<(usize, Vec<T>)> {
        let key = expr as *const Expression<T> as usize;

        if self.memo_table.contains_key(&(key, pos)) {
            return self
                .memo_table
                .get(&(key, pos))
                .expect("Key was not found in the memoization table")
                .clone();
        }

        let result = match expr {
            Expression::Literal(l) => {
                if let Some(s) = self.input.get(pos..) {
                    if s.starts_with(l) {
                        Some((pos + l.len(), vec![T::from_str(l.clone())]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            Expression::Char(ch) => {
                if let Some(first) = self.input.get(pos..) {
                    if first.starts_with(*ch) {
                        Some((pos + ch.len_utf8(), vec![T::from_str(ch.to_string())]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            Expression::CharRange(start_ch, end_ch) => {
                if let Some(slice) = self.input.get(pos..) {
                    let first = slice.chars().next()?;
                    if first >= *start_ch && first <= *end_ch {
                        Some((pos + first.len_utf8(), vec![T::from_str(first.to_string())]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            Expression::Sequence(expressions) => {
                let mut current_pos = pos;
                let mut exprs = vec![];
                for expr in expressions {
                    match self.parse(expr, current_pos) {
                        Some((new_pos, value)) => {
                            current_pos = new_pos;
                            exprs.push(value);
                        }
                        None => return None,
                    }
                }

                Some((current_pos, exprs.iter().flatten().cloned().collect()))
            }

            Expression::OrderedChoice(a, b) => self.parse(a, pos).or_else(|| self.parse(b, pos)),

            Expression::ZeroOrMore(e) => {
                let mut current_pos = pos;
                let mut values = vec![];

                loop {
                    if let Some((new_pos, value)) = self.parse(e, current_pos) {
                        current_pos = new_pos;
                        values.push(value)
                    } else {
                        break;
                    }
                }

                Some((current_pos, values.into_iter().flatten().collect()))
            }

            Expression::Not(e) => {
                if let Some(_) = self.parse(e, pos) {
                    None
                } else {
                    Some((pos, vec![T::from_str("".to_string())]))
                }
            }

            Expression::Action(inner, f) => {
                if let Some((new_pos, value)) = self.parse(inner, pos) {
                    let result = f(value);
                    Some((new_pos, vec![result]))
                } else {
                    None
                }
            }

            Expression::Lazy(f) => self.parse(&f(), pos),
        };

        self.memo_table.insert((key, pos), result.clone());
        result
    }
}

impl<T> Add for Expression<T> {
    type Output = Expression<T>;

    fn add(self, rhs: Self) -> Expression<T> {
        match self {
            Self::Sequence(mut exprs) => {
                exprs.push(rhs);
                Self::Sequence(exprs)
            }

            _ => Self::Sequence(vec![self, rhs]),
        }
    }
}

impl<T> BitOr for Expression<T> {
    type Output = Expression<T>;

    fn bitor(self, rhs: Self) -> Expression<T> {
        Expression::OrderedChoice(Box::new(self), Box::new(rhs))
    }
}
