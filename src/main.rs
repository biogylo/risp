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


#[derive(Debug)]
enum AstNode {
    List(Vec<AstNode>),
    // Int(isize),
    Sym(String),
}

fn take_line(reader: &mut dyn BufRead) -> Option<String> {
    print!("user>");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    let char_count = reader.read_line(&mut line).expect("For now, lets assume there is a line");
    if char_count == 0 {
        // EOF reached
        None
    } else {
        Some(line.trim().into())
    }
}

fn read(line: &str) -> Result<AstNode, Vec<String>> {
    // Find opening and closing parens
    let trimmed_line = line.trim();
    let chars = line.into_boxed_bytes();
    let has_opening_paren = chars.first().unwrap() == b'(';
    if !has_opening_paren {
        // There is a first token, it could be Sym, or another List. Lets see

        if let Some((token, rest)) = trimmed_line.split_once(" ") {
            let mut list_elements = vec![AstNode::Sym(token.into())];

        } else {
            assert!(!trimmed_line.contains(")"))
            AstNode::Sym(trimmed_line.into())
        }
        if trimmed_line.chars().all(|c| c != '(' && c != ')' && !c.is_whitespace()) {
            // Valid symbol
            return Ok(AstNode::Sym(trimmed_line.into()));
        }
        if let Some(the_int) = trimmed_line.parse::<isize>() {
            return Ok(AstNode::Int(the_int));
        } else {
            return Err(vec![format!("Unable to parse expression -> {:}", trimmed_line)]);
        }
    } else {
        // Since there is an opening parenthesis, we will look until we find the matching closing parenthesis
        let remaining = chars[1..-1];
        // An S expression, must make list from it
        // (a (b c) d)
        read();
    };
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
