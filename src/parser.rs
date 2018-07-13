use combine::*;
use combine_language::*;
use combine::char::{alpha_num, letter, string, spaces, char};
use combine::primitives::Error;

use ast::*;
use ast::{Expr::*, Statement::*};

const KEYWORDS: &[&str] = &["if", "then", "else", "fn", "let"];

/// Language environment: provides lexing, comment support and easy expr parsing
pub fn language_env<'a, I: 'a>() -> LanguageEnv<'a, I>
    where I: Stream<Item = char>
{
    LanguageEnv::new(LanguageDef {
        ident: Identifier {
            start: letter(),
            rest: alpha_num().or(char('_')),
            reserved: KEYWORDS.iter().map(|x| (*x).into()).collect(),
        },
        op: Identifier {
            start: satisfy(|c| "+-*/=".chars().any(|x| x == c)),
            rest: satisfy(|c| "+-*/=".chars().any(|x| x == c)),
            reserved: ["+", "-", "*", "/", "="].iter().map(|x| (*x).into()).collect()
        },
        comment_start: string("/*").map(|_| ()),
        comment_end: string("*/").map(|_| ()),
        comment_line: string("//").map(|_| ()),
    })
}

/// Construct a parse error from a string
fn str_error<T, E>(s: &'static str) -> Error<T, E> {
    Error::Message(s.into())
}

/// Statement parser
pub fn statement_fn<I>(
    input: I,
    env: &LanguageEnv<I>,
) -> ParseResult<Statement, I>
where
    I: Stream<Item=char>
{
    let let_parser = (
        env.reserved("let"),
        env.identifier(),
        env.reserved_op("="),
        expr()
    ).map(|(_, ident, _, expr)| Statement::SLet(ident, expr));

    let_parser.or(expr().map(Statement::SExpr)).parse_stream(input)
}

/// Statement parser
pub fn statement<'a, I: Stream<Item=char> + 'a>() -> impl Parser<Input=I, Output=Statement> {
    parser(|inp| statement_fn(inp, &language_env()))
}

/// Construct an expression from an LHS, an operator and an RHS
fn op(l: Expr, o: &'static str, r: Expr) -> Expr {
    Op(Box::new(l), o, Box::new(r))
}

/// Expression parser
pub fn expr_fn<I>(input: I, lang_env: &LanguageEnv<I>) -> ParseResult<Expr, I>
    where I: Stream<Item=char>
{
    let lex_char = |c| lang_env.lex(char(c));

    // Expression blocks { e1; e2; e3 }
    let expr_list = sep_by(statement().or(string("").map(|_| SExpr(Unit))), lex_char(';'))
        .and_then(|mut stmts: Vec<_>| {
            match stmts.pop() {
                Some(SExpr(terminal_expr)) => Ok((stmts, Some(terminal_expr))),
                Some(_) =>
                    Err(str_error("Expression blocks can't be terminated with statements")),
                None => unreachable!("Expression block parser should at least produce ()"),
            }
        });

    let expr_block = between(lex_char('{'), lex_char('}'), expr_list)
        .map(|(stmts, opt_expr)| {
            Expr::Block(stmts, opt_expr.map(Box::new))
        });

    // Simple terms and operators: var_name, x + y * z, etc
    // FIXME: precedence for other operators
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

    // Function calls
    let fn_call = (
        lang_env.identifier(),
        between(lex_char('('), lex_char(')'), sep_end_by(expr(), lex_char(',')))
    ).map(|(function, args)| {
        FnCall {
            function,
            args,
        }
    });

    expr_block
        // FIXME: `try` required to avoid confusing `ident` with `ident(args..)`
        // Feels like a hack, could probably do something better.
        .or(try(fn_call))
        .or(prim_expr)
        .parse_stream(input)
}

/// Expression parser
pub fn expr<'a, I: Stream<Item=char> + 'a>() -> impl Parser<Input=I, Output=Expr> {
    parser(|inp| expr_fn(inp, &language_env()))
}

// Function parser
pub fn function_fn<I>(input: I, env: &LanguageEnv<I>) -> ParseResult<Function, I>
    where I: Stream<Item=char>
{
    let expr = parser(|inp| expr_fn(inp, env));

    // TODO: proper type parser
    let func_arg = (
        env.identifier().skip(env.lex(char(':'))),
        env.identifier()
    );

    let mut func = (
        env.lex(env.reserved("fn")),
        env.identifier(),
        between(env.lex(char('(')), env.lex(char(')')), sep_by(func_arg, env.lex(char(',')))),
        expr
    ).map(|(_, name, argument_list, body)| {
        Function {
            name,
            argument_list,
            body
        }
    });

    func.parse_stream(input)
}

/// Function parser
pub fn function<'a, I: Stream<Item=char> + 'a>() -> impl Parser<Input=I, Output=Function> {
    parser(|inp| function_fn(inp, &language_env()))
}
