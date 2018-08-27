use combine::*;
use combine_language::*;
use combine::char::{alpha_num, letter, string, spaces, char};
use combine::primitives::Error;

use ast::*;
use ast::{Expr::*, Statement::*};

const KEYWORDS: &[&str] = &["if", "then", "else", "fn", "let", "true", "false"];

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
            reserved: ["+", "-", "*", "/", "=", "<", ">"].iter().map(|x| (*x).into()).collect()
        },
        comment_start: string("/*").map(|_| ()),
        comment_end: string("*/").map(|_| ()),
        comment_line: string("//").map(|_| ()),
    })
}

/// Construct a parse error from a string
pub fn str_error<T, E>(s: &'static str) -> Error<T, E> {
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

    // TODO: more literals
    let string_literal = lang_env.string_literal().map(StringLit);

    let int_literal = lang_env.integer().map(IntLit);

    let bool_literal =
        lang_env.reserved("true").map(|_| BoolLit(true)).or(
            lang_env.reserved("false").map(|_| BoolLit(false))
        );

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

    // If expressions
    // FIXME: statement if (else-less)
    let if_expr = (
        lang_env.reserved("if"),
        expr().skip(look_ahead(lex_char('{'))),
        expr(),
        lang_env.reserved("else").skip(look_ahead(lex_char('{'))),
        expr(),
    ).and_then(|(_, cond, e1, _, e2)| {
        match (&e1, &e2) {
            (Block(..), Block(..)) => {
                Ok(If(Box::new(cond), Box::new(e1), Box::new(e2)))
            }
            _ => {
                Err(str_error("Expected braces around if expression body: if .. { .. } else { .. }"))
            }
        }
    });

    let ident = lang_env.identifier().map(Id);

    // FIXME: I have a cold and I'm not sure if this precedence magic is right
    let term = expr_block
        // FIXME: `try` required to avoid confusing `ident` with `ident(args..)`
        // Feels like a hack, could probably do something better.
        .or(try(fn_call))
        .or(if_expr)
        .or(bool_literal)
        .or(ident)
        .or(string_literal)
        .or(int_literal);

    expression_parser(term, op_parser, op).parse_stream(input)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bool_lit() {
        assert_eq!(expr().parse_stream("true").unwrap().0, BoolLit(true));
    }
}
