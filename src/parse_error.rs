use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
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
}
