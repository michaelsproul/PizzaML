use std::io::{self, Write};
use ast::*;
use ast::{Expr::*, Statement::*};

pub fn translate_expression<W: Write>(e: &Expr, o: &mut W) -> io::Result<()> {
    match *e {
        Id(ref ident) => write!(o, "{}", ident)?,
        Op(ref lhs, op, ref rhs) => {
            translate_expression(lhs, o)?;
            write!(o, "{}", op)?;
            translate_expression(rhs, o)?;
        }
        FnCall { ref function, ref args } => {
            write!(o, "{}", function)?;
            for arg in args {
                write!(o, " ")?;
                translate_expression(arg, o)?;
            }
        }
        Unit => write!(o, "()")?,
        Block(ref stmts, ref terminal) => {
            if !stmts.is_empty() {
                write!(o, "let\n")?;
                for stmt in stmts {
                    translate_statement(stmt, o)?;
                }
                write!(o, "in ")?;
            }

            match terminal {
                Some(e) => translate_expression(e, o)?,
                None => translate_expression(&Unit, o)?,
            }

            if !stmts.is_empty() {
                write!(o, "\nend")?;
            }
        }
        If(..) => unimplemented!(),
    }
    Ok(())
}

pub fn translate_expression_to_str(e: &Expr) -> Result<String, io::Error> {
    let mut buffer = Vec::new();
    translate_expression(e, &mut buffer)?;
    let result = String::from_utf8(buffer).expect("translated ML should be valid utf-8");
    Ok(result)
}

pub fn translate_statement<W: Write>(s: &Statement, o: &mut W) -> io::Result<()> {
    match *s {
        SLet(ref ident, ref body) => {
            write!(o, "val {} = (", ident)?;
            translate_expression(body, o)?;
            write!(o, ");\n")?;
        }
        SExpr(ref expr) => {
            write!(o, "val _ = (")?;
            translate_expression(expr, o)?;
            write!(o, ");\n")?;
        }
    }
    Ok(())
}

// TODO: polymorphic types
pub fn translate_function<W: Write>(f: &Function, o: &mut W) -> io::Result<()> {
    write!(o, "fun {} ", f.name)?;
    for &(ref name, ref ty) in &f.argument_list {
        write!(o, "({}: {}) ", name, ty)?;
    }
    write!(o, "= ")?;
    translate_expression(&f.body, o)?;
    write!(o, ";")?;
    Ok(())
}

pub fn translate_function_to_str(f: &Function) -> Result<String, io::Error> {
    let mut buffer = Vec::new();
    translate_function(f, &mut buffer)?;
    let result = String::from_utf8(buffer).expect("translated ML should be valid utf-8");
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use combine::Parser;
    use parser::*;

    #[test]
    fn simple_block() {
        let e = expr().parse("{ e1; e2; e3 }").unwrap().0;
        let translated = translate_expression_to_str(&e).unwrap();

        assert_eq!(
            translated,
r##"
let
val _ = (e1);
val _ = (e2);
in e3
end
"##.trim()
        );
    }

    #[test]
    fn function_fn_call() {
        let f = Function {
            name: "foo".into(),
            argument_list: vec![("x".into(), "Int".into())],
            body: FnCall {
                function: "bar".into(),
                args: vec![Unit, Unit, Unit]
            },
        };
        let ml_func = translate_function_to_str(&f).unwrap();

        assert_eq!(&ml_func, "fun foo (x: Int) = bar () () ();");
    }
}
