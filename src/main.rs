#![feature(try_trait_v2)]
#![feature(byte_slice_trim_ascii)]
#![feature(slice_split_once)]

use std::{env, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use jirsp::parse_error::ParseError;
use jirsp::tokenize::AstNode;
use jirsp::tokenize::AstToken::Parsed;
use jirsp::tokenize::tokenize;

use crate::result::CliError::TooManyArguments;
use crate::result::RispError;
use crate::result::RispResult;

mod result;

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

fn take_line(reader: &mut dyn BufRead) -> Option<Box<[u8]>> {
    print!("user>");
    io::stdout().flush().unwrap();
    let mut line: Vec<u8> = vec![];
    let char_count = reader.read_until(b'\n', &mut line).expect("For now, lets assume there is a line");
    assert!(line.is_ascii(), "We should only be using ascii here, the line: {:?}", line);
    if char_count == 0 {
        // EOF reached
        None
    } else {
        let trimmed = line.trim_ascii();
        Some(trimmed.into())
    }
}


fn read(line: &[u8]) -> Result<AstNode, ParseError> {
    if let Parsed(node) = tokenize(line)? {
        Ok(node)
    } else {
        Err(ParseError::NotAnSExpression)
    }
}

fn eval(node: AstNode) -> AstNode { node }

fn print(node: AstNode) {
    println!("{:?}", node);
    io::stdout().flush().unwrap();
}


fn main() -> RispResult<()> {
    let arguments: Vec<String> = env::args().collect();
    let mut handle: Box<dyn BufRead> = get_input_handle(&arguments[1..])?;
    while let Some(line) = take_line(handle.as_mut()) {
        let parsed = read(&line);
        match parsed {
            Ok(parsed) => {
                let result = eval(parsed);
                let _printed = print(result);
            }
            Err(parse_error) => { println!("{:}", parse_error) }
        }
    };
    // EOF Reached
    RispResult::_ok()
}