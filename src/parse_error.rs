use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("cannot parse empty buffer")]
    CannotParseEmpty,
    #[error("missing left parenthesis in S expression")]
    MissingLeftParenthesis,
    #[error("missing right parenthesis in S expression")]
    MissingRightParenthesis,
    #[error("unparseable empty expression passed in")]
    EmptyExpression,
    #[error("forbidden char in symbol ({0})")]
    ForbiddenCharInSymbol(char),
    #[error("the given expression was not an S expression")]
    NotAnSExpression,
    #[error("the given atom is not a valid number ({0})")]
    CannotParseNumber(String),
    #[error("a double-quote string was opened, but not matched")]
    MissingDoubleQuote,
    #[error("a double-quote string was closed, but that wasn't the end of it")]
    StringDidntEnd,
}
