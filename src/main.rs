extern crate combine;
extern crate combine_language;

use combine::*;
use combine_language::*;

use combine::char::{alpha_num, letter, string, spaces, char};
use combine::primitives::Error;

use self::Expr::*;
use self::Statement::*;

#[derive(PartialEq, Debug)]
enum Statement<E> {
    // Let binding
    SLet(String, E),
    // Probably side-effectual expression <expr>;
    SExpr(E),
    // TODO: Assignment?
}

#[derive(PartialEq, Debug)]
enum Expr {
    Id(String),
    Op(Box<Expr>, &'static str, Box<Expr>),
    Block(Vec<Statement<Expr>>, Option<Box<Expr>>)
}

fn op(l: Expr, o: &'static str, r: Expr) -> Expr {
    Op(Box::new(l), o, Box::new(r))
}

fn str_error<T, E>(s: &'static str) -> Error<T, E> {
    Error::Message(s.into())
}

#[derive(PartialEq, Debug)]
struct Function {
    name: String,
    argument_list: Vec<(String, String)>,
    body: Expr,
}

fn main() {
    let keywords = ["if", "then", "else", "fn", "let"];

    let env = LanguageEnv::new(LanguageDef {
        ident: Identifier {
            start: letter(),
            rest: alpha_num().or(char('_')),
            reserved: keywords.iter().map(|x| (*x).into()).collect(),
        },
        op: Identifier {
            start: satisfy(|c| "+-*/=".chars().any(|x| x == c)),
            rest: satisfy(|c| "+-*/=".chars().any(|x| x == c)),
            reserved: ["+", "-", "*", "/", "="].iter().map(|x| (*x).into()).collect()
        },
        comment_start: string("/*").map(|_| ()),
        comment_end: string("*/").map(|_| ()),
        comment_line: string("//").map(|_| ()),
    });

    // Statement parser (parametrised by an expr parser)
    fn statement_fn<I>(
        input: I,
        env: &LanguageEnv<I>,
        expr_fn: impl Fn(I, &LanguageEnv<I>) -> ParseResult<Expr, I>)
    -> ParseResult<Statement<Expr>, I>
    where
        I: Stream<Item=char>
    {
        let expr = parser(|inp| expr_fn(inp, env));
        let let_parser = (
            env.reserved("let"),
            env.identifier(),
            env.reserved_op("="),
            expr.clone()
        ).map(|(_, ident, _, expr)| Statement::SLet(ident, expr));

        let_parser.or(expr.map(Statement::SExpr)).parse_stream(input)
    }

    // Expression parser
    fn expr_fn<I>(input: I, lang_env: &LanguageEnv<I>) -> ParseResult<Expr, I>
        where I: Stream<Item=char>
    {
        let lex_char = |c| lang_env.lex(char(c));

        let statement = parser(|inp| statement_fn(inp, lang_env, expr_fn::<I>));
        let expr_list = (
            sep_by(statement, lex_char(';')),
            optional(lex_char(';')),
        ).and_then(|(mut stmts, final_semicolon): (Vec<_>, _)| {
            // Extract last expression like e3 in { e1; e2; e3 }
            if final_semicolon.is_none() {
                match stmts.pop() {
                    Some(SExpr(terminal_expr)) => Ok((stmts, Some(terminal_expr))),
                    Some(_) =>
                        Err(str_error("Expression blocks can't be terminated with statements")),
                    None => Ok((stmts, None))
                }
            } else {
                Ok((stmts, None))
            }
        });

        let expr_block = between(lex_char('{'), lex_char('}'), expr_list)
            .map(|(stmts, opt_expr)| {
                Expr::Block(stmts, opt_expr.map(Box::new))
            });

        let op_parser = string("+").or(string("*"))
            .map(|op| {
                let prec = match op {
                    "+" => 6,
                    "*" => 7,
                    _ => unreachable!()
                };
                (op, Assoc { precedence: prec, fixity: Fixity::Left })
            })
            .skip(spaces());

        let term = lang_env
            .identifier()
            .map(Id)
            .skip(spaces());

        let prim_expr = expression_parser(term, op_parser, op);

        expr_block
            .or(prim_expr)
            .parse_stream(input)
    }

    let mut expr = parser(|inp| expr_fn(inp, &env));

    // TODO: proper type parser
    let func_arg = (
        env.identifier().skip(env.lex(char(':'))),
        env.identifier()
    );

    let mut func = (
        env.lex(env.reserved("fn")),
        env.identifier(),
        between(env.lex(char('(')), env.lex(char(')')), sep_by(func_arg, env.lex(char(',')))),
        expr.clone()
    ).map(|(_, name, argument_list, body)| {
        Function {
            name,
            argument_list,
            body
        }
    });

    println!("Testing the expression parser:");
    println!("{:#?}", expr.parse(State::new("{{ hello_world + this_is_cool * wowza; x }; { x; y }}")));

    let example = "fn test_function ( arg1 : Type1 , arg2: Type2 ) { arg1 + arg2 * arg2; arg2 }";

    println!("Testing the function parser:");
    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }

    let example =
r##"fn test_function(x: Int, y: Int) {
        let z1 = x + y * x;
        let z2 = z1 * z1;
        z2;
        z1 + z2
    }
"##;

    println!("Testing the function parser:");
    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }

    // This should error because the terminal expression is a let
    let example =
r##"fn test_function(x: Int, y: Int) {
        let z1 = x + y * x;
        let z2 = z1 * z1;
        z2;
        let y = z1 + z2
    }
"##;

    println!("Testing the function parser:");
    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }
}
