#![feature(try_trait_v2)]
#![feature(byte_slice_trim_ascii)]
#![feature(slice_split_once)]

use std::{env, io};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;

use jirsp::eval::{eval, EvalError, GlobalNamespace};
use jirsp::parse_error::ParseError;
use jirsp::tokenize::{AstNode, tokenize, Value};
use jirsp::tokenize::AstToken::Parsed;

use crate::result::RispError;

mod result;


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
            Err(RispError::TooManyArguments(arguments.len()))
        }
    }
}

fn read(reader: &mut dyn BufRead) -> Option<Box<[u8]>> {
    print!("user>");
    io::stdout().flush().unwrap();
    let mut line: Vec<u8> = vec![];
    let char_count = reader.read_until(b'\n', &mut line).expect("For now, lets assume there is a line");
    if char_count == 0 {
        // EOF reached
        None
    } else {
        let trimmed = line.trim_ascii();
        Some(trimmed.into())
    }
}

fn parse(line: &[u8]) -> Result<AstNode, ParseError> {
    if let Parsed(node) = tokenize(line)? {
        Ok(node)
    } else {
        Err(ParseError::NotAnSExpression)
    }
}

fn print(eval_result: &Result<Value, impl Error>) {
    match eval_result {
        Ok(ref value) => println!("{}", value),
        Err(ref parse_error) => println!("{}", parse_error)
    };
}

fn print_debug(eval_result: &Result<AstNode, impl Error>) {
    match eval_result {
        Ok(ref ast_node) => println!("{:?}", ast_node),
        Err(ref parse_error) => println!("{}", parse_error)
    };
}

fn risp(mut input_handle: Box<dyn BufRead>) {
    let mut namespace = GlobalNamespace::default();
    while let Some(line) = read(&mut input_handle) {
        let result: Result<AstNode, ParseError> = parse(&line);
        print_debug(&result);
        let Ok(node) = result else {
            continue;
        };
        let result: Result<Value, EvalError> = eval(&node, &mut namespace);
        print(&result)
    };
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().collect();
    match get_input_handle(&arguments[1..]) {
        Err(error) => {
            println!("{}", error);
            ExitCode::FAILURE
        }
        Ok(input_handle) => {
            risp(input_handle);
            ExitCode::SUCCESS
        }
    }
}