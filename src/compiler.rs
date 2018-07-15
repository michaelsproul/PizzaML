//! Combine parsing, translation and compilation with a Standard ML compiler
use std::io;
use combine::*;
use combine::primitives::Error;

use parser;
use translator::*;

pub fn parse_and_translate<I, W>(source_code: I, output: &mut W) -> Result<(), ParseError<I>>
    where
        I: Stream<Item = char>,
        W: io::Write,
{
    parser::function()
        .and_then(|func| {
            translate_function(&func, output).map_err(Error::from)
        })
        .parse(source_code)
        // FIXME: check there's nothing left in the stream
        .map(|_| ())
}
