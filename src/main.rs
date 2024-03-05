#![feature(try_trait_v2)]

mod result;

use std::{env, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::result::CliError::TooManyArguments;
use crate::result::RispError;
use crate::result::RispResult;


const USAGE: &'static str = "Usage:
    risp <filepath>
        Interpret risp from a file
    risp
        Repl
";

fn get_input_handle(arguments: &[String]) -> Result<Box<dyn BufRead>, RispError> {
    match arguments {
        [] => {
            let stdin_lock = io::stdin().lock();
            Ok(Box::new(stdin_lock))
        }
        [filepath] => {
            let f = File::open(filepath).map_err(RispError::from)?;
            Ok(Box::new(BufReader::new(f)))
        }
        _ => {
            Err(RispError::CliError(TooManyArguments(arguments.len())))
        }
    }
}


fn read(reader: &mut dyn BufRead) -> String {
    print!("user>");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).expect("For now, lets assume there is a line");
    line.trim().into()
}

fn eval(line: &str) -> String { line.into() }

fn print(line: &str) -> String {
    println!("{line}");
    io::stdout().flush().unwrap();
    line.into()
}

fn main() -> RispResult<()> {
    let arguments: Vec<String> = env::args().collect();
    let mut handle: Box<dyn BufRead> = get_input_handle(&arguments[1..])?;
    loop {
        let line = read(handle.as_mut());
        let result = eval(&line);
        let _printed = print(&result);
    }
}
