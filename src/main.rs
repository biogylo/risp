#![feature(try_trait_v2)]

mod result;

use std::{env, io};
use std::fs::File;
use std::io::Read;
use crate::result::CliError::TooManyArguments;
use crate::result::RispError;
use crate::result::RispResult;


const USAGE: &'static str = "Usage:
    risp <filepath>
        Interpret risp from a file
    risp
        Repl
";

fn get_input_handle(arguments: &[String]) -> Result<Box<dyn Read>, RispError> {
    match arguments {
        [] => {
            let stdin_lock = io::stdin().lock();
            Ok(Box::new(stdin_lock))
        }
        [filepath] => {
            let f = File::open(filepath).map_err(RispError::from)?;
            Ok(Box::new(f))
        }
        _ => {
            Err(RispError::CliError(TooManyArguments(arguments.len())))
        }
    }
}


fn main() -> RispResult<()> {
    let arguments: Vec<String> = env::args().collect();
    let mut handle: Box<dyn Read> = get_input_handle(&arguments[1..])?;
    let mut buffer = Vec::new();
    let total_read = handle.read_to_end(&mut buffer)?;
    println!("Read a total of {total_read} chars.");
    RispResult::ok()
}
