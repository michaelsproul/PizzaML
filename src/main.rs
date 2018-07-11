extern crate combine;
extern crate combine_language;

mod ast;
mod parser;

use combine::{Parser, State};
use parser::{expr_fn, function_fn};

fn main() {
    let env = parser::language_env();
    let mut expr = combine::parser(|inp| expr_fn(inp, &env));
    let mut func = combine::parser(|inp| function_fn(inp, &env));

    println!("Testing the expression parser:");
    println!("{:#?}", expr.parse(State::new("{{ hello_world + this_is_cool * wowza; x }; { x; y }}")));

    println!("Testing the function parser (#1):");
    let example = "fn test_function ( arg1 : Type1 , arg2: Type2 ) { arg1 + arg2 * arg2; arg2 }";
    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }

    println!("Testing the function parser (#2):");
    let example =
r##"fn test_function(x: Int, y: Int) {
        let z1 = x + y * x;
        let z2 = z1 * z1;
        z2;
        z1 + z2
    }
"##;

    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }

    println!("Testing the expr parser (#3):");
    let example = "foo(bar)";
    println!("{:#?}", expr.parse(State::new(example)));

    println!("Testing the function parser (#4):");
    let example =
r##"fn test_function(x: Int, y: Int) {
        let z1 = x + y * x;
        let z2 = some_other_function(x, y, z);
        z2;
        z1 + z2
    }
"##;

    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }

    // This should error because the terminal expression is a let
    println!("Testing the function parser (#5):");
    let example =
r##"fn test_function(x: Int, y: Int) {
        let z1 = x + y * x;
        let z2 = z1 * z1;
        z2;
        let y = z1 + z2
    }
"##;

    match func.parse(State::new(example)) {
        Ok(res) => println!("{:#?}", res),
        Err(err) => println!("{}", err),
    }
}
