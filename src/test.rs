use combine::*;

use ast::Expr::*;
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
