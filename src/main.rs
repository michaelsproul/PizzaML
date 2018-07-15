extern crate pizza_ml;
extern crate combine;

use std::io::Read;
use std::fs::File;
use std::env;

use combine::State;
use pizza_ml::compiler::parse_and_translate;

// FIXME: remove unwraps
fn main() {
    let args: Vec<_> = env::args().collect();
    let src_file_name = &args[1];

    let mut src_file = File::open(src_file_name).unwrap();

    // FIXME: use streaming
    let mut src = String::new();
    src_file.read_to_string(&mut src).unwrap();

    let mut output_file = File::create("output.sml").unwrap();

    parse_and_translate(State::new(&src[..]), &mut output_file).unwrap();
}
