use std::io;
use thiserror::Error;


const USAGE: &'static str = "Usage:
    risp <filepath>
        Interpret risp from a file
    risp
        Repl
";

#[derive(Error, Debug)]
pub enum RispError {
    #[error("too many arguments provided: {0}\n{}", USAGE)]
    TooManyArguments(usize),
    #[error("unable to open file {0}")]
    UnableToOpenFile(#[from] io::Error),
}