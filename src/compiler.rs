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
        // Check that all input was consumed
        .and_then(|(_, mut stream)| {
            if stream.uncons().is_err() {
                Ok(())
            } else {
                Err(ParseError::new(
                    stream.position(),
                    parser::str_error("Failed to parse all input"),
                ))
            }
        })
}
