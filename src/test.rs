use combine::*;

use ast::{Expr::*, Statement::*};
use parser::*;

#[test]
fn expr_fn_call() {
    // TODO: short-hand syntax for constructing terms
    // TODO: short-hand for complete parses (empty string left-over)
    assert_eq!(
        expr().parse("add(x, y)"),
        Ok((FnCall {
            function: "add".into(),
            args: vec![Id("x".into()), Id("y".into())],
        }, ""))
    );
}

#[test]
fn expr_block_empty() {
    assert_eq!(expr().parse("{}").unwrap().0, Block(vec![], Some(Box::new(Unit))));
}

#[test]
fn expr_block_with_terminal() {
    assert_eq!(
        expr().parse("{ e1; e2; e3 }"),
        Ok((Block(
            vec![SExpr(Id("e1".into())), SExpr(Id("e2".into()))],
            Some(Box::new(Id("e3".into()))),
        ), ""))
    )
}

#[test]
fn expr_block_no_terminal() {
    assert_eq!(
        expr().parse("{ e1; e2; e3; }").unwrap().0,
        Block(
            vec![SExpr(Id("e1".into())), SExpr(Id("e2".into())), SExpr(Id("e3".into()))],
            Some(Box::new(Unit))
        )
    );
}
