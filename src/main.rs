#![feature(try_trait_v2)]
#![feature(byte_slice_trim_ascii)]
#![feature(slice_split_once)]

mod result;

use std::{env, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use thiserror::Error;
use jirsp::parse_error::ParseError;
use crate::ParseError::EmptyExpression;
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

fn parse_s_expression(s_expr: &[u8]) -> Result<Vec<AstNode>, ParseError> {
    // Assume this is an S expression with matched parens
    // First, ensure first is a parenthesis
    let trimmed = s_expr.trim_ascii();
    if trimmed.is_empty() {
        return Err(EmptyExpression);
    }
    // Check for left and right parenthesis
    trimmed.first().filter(|&c| c == b'(').ok_or_else(|| ParseError::MissingLeftParenthesis)?;
    trimmed.last().filter(|&c| c == b')').ok_or_else(|| ParseError::MissingRightParenthesis)?;

    let start = 1;
    let end = trimmed.len() - 2;
    // Parse what's within as a list
    let list_expr = &trimmed[start..end];
    return parse_list(list_expr);
}


fn parse_list(list_expr: &[u8]) -> Result<Vec<AstNode>, ParseError> {
    let trimmed = list_expr.trim_ascii();
    let mut the_list = vec![];
    let mut remaining_buffer = trimmed.into_boxed_slice();
    while !remaining_buffer.is_empty() {
        let Some(first_char) = trimmed.first() else {
            // Empty list
            return Ok(vec![]);
        };
        // Please god, let it not be another S expression
        if first_char != b'(' {

            // God is on our side, we can parse as symbol
            // Read until whitespace

        }

    }
    if trimmed.is_empty() {
    }
    // Either we start with a symbol, or another s expr
    trimmed.first().expect("We ensured the list isn't empty")
    todo!()
}

fn parse_symbol() {
    // Assume this is a symbol
}

fn read(line: &[u8]) -> Result<AstNode, Vec<String>> {
    // Find opening and closing parens
    line.trim().into_boxed_bytes();
    todo!()
}

fn eval(node: AstNode) -> AstNode { node }

fn print(node: AstNode) -> String {
    println!("{:?}", node);
    io::stdout().flush().unwrap();
    line.into()
}


fn main() -> RispResult<()> {
    let arguments: Vec<String> = env::args().collect();
    let mut handle: Box<dyn BufRead> = get_input_handle(&arguments[1..])?;
    while let Some(line) = take_line(handle.as_mut()) {
        let parsed: AstNode = read(&line);
        let result = eval(&parsed);
        let _printed = print(&result);
    };
    // EOF Reached
    RispResult::_ok()
}

fn main() {}