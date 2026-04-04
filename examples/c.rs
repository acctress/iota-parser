use iota::{
    Expression, IotaParser, ParseValue, action, alpha, alphanum, between, ch, ignore, lazy, lit,
    one_or_more, optional, symbol, values, whitespace, ws, zero_or_more,
};

#[derive(Debug, Clone)]
enum CType {
    Int,
    Void,
}

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Ident(String),
    BinOp(Box<Expr>, String, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    VarDecl(CType, String, Expr),
    Return(Expr),
    FnDecl(CType, String, Vec<(CType, String)>, Vec<Stmt>),
}

/* okay no we can define our ParseValue for iota */
#[derive(Debug, Clone)]
enum Node {
    Raw(String),
    Expression(Expr),
    Statement(Stmt),
    Params(Vec<(CType, String)>),
    Stmts(Vec<Stmt>),
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

/* now define our parse helper functions for C-lang */

fn nodes(vals: Vec<Node>) -> Vec<Node> {
    values(vals)
        .into_iter()
        .filter(|v| match v {
            Node::Expression(_) => true,
            _ => !v.as_str().is_empty(),
        })
        .collect()
}

fn identifier() -> Expression<Node> {
    action(alpha() + zero_or_more(alphanum()), |vals| {
        Node::Raw(values(vals).iter().map(|v| v.as_str()).collect())
    })
}

fn number() -> Expression<Node> {
    action(one_or_more(|| iota::digit()), |vals| {
        Node::Raw(values(vals).iter().map(|v| v.as_str()).collect())
    })
}

fn ctype() -> Expression<Node> {
    action(lit("int") | lit("void"), |vals| {
        Node::Raw(values(vals)[0].as_str())
    })
}

/* now for the structural parser functions for C-lang */

fn expr() -> Expression<Node> {
    action(
        atom() + optional(|| ignore(ws()) + symbol() + ignore(ws()) + lazy(expr)),
        |vals| {
            let parts = nodes(vals);
            let lhs = parse_atom(&parts[0].as_str());

            if parts.len() == 1 {
                Node::Expression(lhs)
            } else {
                let op = parts[1].as_str();
                let rhs = match &parts[2] {
                    Node::Expression(e) => e.clone(),
                    _ => parse_atom(&parts[2].as_str()),
                };
                Node::Expression(Expr::BinOp(Box::new(lhs), op, Box::new(rhs)))
            }
        },
    )
}

fn atom() -> Expression<Node> {
    number() | identifier()
}

fn parse_atom(s: &str) -> Expr {
    if let Ok(n) = s.parse::<i64>() {
        Expr::Number(n)
    } else {
        Expr::Ident(s.to_string())
    }
}

/* statement parsers */

fn var_decl() -> Expression<Node> {
    action(
        ctype()
            + ignore(ws())
            + identifier()
            + ignore(ws())
            + ignore(lit("="))
            + ignore(ws())
            + expr()
            + ignore(lit(";")),
        |vals| {
            let p = values(vals);
            let t = match p[0].as_str().as_str() {
                "int" => CType::Int,
                _ => CType::Void,
            };

            let name = p[1].as_str();
            let ex = match &p[2] {
                Node::Expression(e) => e.clone(),
                _ => Expr::Number(0),
            };

            Node::Statement(Stmt::VarDecl(t, name, ex))
        },
    )
}

fn return_stmt() -> Expression<Node> {
    action(
        ignore(lit("return")) + ignore(ws()) + expr() + ignore(lit(";")),
        |vals| {
            let p = values(vals);
            let ex = match &p[0] {
                Node::Expression(e) => e.clone(),
                _ => Expr::Number(0),
            };

            Node::Statement(Stmt::Return(ex))
        },
    )
}

fn param() -> Expression<Node> {
    action(ctype() + ignore(ws()) + identifier(), |vals| {
        let p = values(vals);
        let t = match p[0].as_str().as_str() {
            "int" => CType::Int,
            _ => CType::Void,
        };

        Node::Raw(format!("{}:{}", p[0].as_str(), p[1].as_str()))
    })
}

fn params() -> Expression<Node> {
    action(
        between(
            ch('('),
            ch(')'),
            optional(|| param() + zero_or_more(ignore(lit(",")) + ignore(ws()) + param())),
        ),
        |vals| {
            let parts = values(vals)
                .into_iter()
                .filter(|v| !v.as_str().is_empty())
                .collect::<Vec<_>>();
            let parsed = parts
                .iter()
                .map(|v| {
                    let s = v.as_str();
                    let mut split = s.splitn(2, ':');
                    let t = match split.next().unwrap_or("") {
                        "int" => CType::Int,
                        _ => CType::Void,
                    };
                    let name = split.next().unwrap_or("").to_string();
                    (t, name)
                })
                .collect();
            Node::Params(parsed)
        },
    )
}

fn stmt() -> Expression<Node> {
    var_decl() | return_stmt()
}

fn body() -> Expression<Node> {
    action(
        between(
            ch('{'),
            ch('}'),
            zero_or_more(ignore(ws()) + lazy(stmt) + ignore(ws())),
        ),
        |vals| {
            let stmts = values(vals)
                .into_iter()
                .filter_map(|v| match v {
                    Node::Statement(s) => Some(s),
                    _ => None,
                })
                .collect();
            Node::Stmts(stmts)
        },
    )
}

fn fn_decl() -> Expression<Node> {
    action(
        ctype() + ignore(ws()) + identifier() + params() + ignore(ws()) + body(),
        |vals| {
            let parts = values(vals);
            let t = match parts[0].as_str().as_str() {
                "int" => CType::Int,
                _ => CType::Void,
            };
            let name = parts[1].as_str();
            let params = match &parts[2] {
                Node::Params(p) => p.clone(),
                _ => vec![],
            };
            let stmts = match &parts[3] {
                Node::Stmts(s) => s.clone(),
                _ => vec![],
            };
            Node::Statement(Stmt::FnDecl(t, name, params, stmts))
        },
    )
}

fn main() {
    let sources = [
        "int x = 5;",
        "int x = a + b;",
        "int add(int a, int b) { int x = a + b; return x; }",
    ];

    for source in sources {
        println!("input:  {source}");
        let mut parser = IotaParser::<Node>::new(source.to_string());
        let expr = if source.contains('(') {
            fn_decl()
        } else {
            var_decl()
        };

        match parser.parse(&expr, 0) {
            Some((_, vals)) => println!("output: {:#?}\n", vals[0]),
            None => println!("Parse failed\n"),
        }
    }
}
