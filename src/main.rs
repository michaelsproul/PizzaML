extern crate combine;
extern crate combine_language;

use combine::*;
use combine_language::*;

use combine::char::{alpha_num, letter, string, spaces, char};

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

// Expression parser
fn op(l: Expr, o: &'static str, r: Expr) -> Expr {
    Op(Box::new(l), o, Box::new(r))
}

fn id(s: &str) -> Expr {
    Id(String::from(s))
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
            // FIXME: statement parser is too greedy here and eats the last expr
            // a different soln will be required
            sep_end_by(statement, lex_char(';')),
            optional(parser(|inp| expr_fn::<I>(inp, lang_env))),
        );

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
}
