use std::fmt::{self, Write};

use ast::*;
use ast::Expr::*;

// TODO: polymorphic types
pub fn translate_function(f: &Function, mut o: impl Write) -> fmt::Result {
    write!(o, "fun {} ", f.name)?;
    for &(ref name, ref ty) in &f.argument_list {
        write!(o, "({}: {}) ", name, ty)?;
    }
    write!(o, "= ")?;
    write!(o, "()")?;
    write!(o, ";")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn translate_fn() {
        let f = Function {
            name: "foo".into(),
            argument_list: vec![("x".into(), "Int".into())],
            body: Unit,
        };
        let mut ml_func = String::new();
        translate_function(&f, &mut ml_func).unwrap();

        assert_eq!(&ml_func, "fun foo (x: Int) = ();");
    }
}
