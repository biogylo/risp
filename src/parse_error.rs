use thiserror::Error;


#[derive(Error, Debug)]
pub enum SymbolParseError {
    #[error("forbidden char in symbol {0}")]
    ForbiddenCharInSymbol(u8)
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("missing left parenthesis in S expression")]
    MissingLeftParenthesis,
    #[error("missing right parenthesis in S expression")]
    MissingRightParenthesis,
    #[error("unparseable empty expression passed in")]
    EmptyExpression,
    #[error("unable to parse symbol {0}")]
    SymbolParseError(#[from] SymbolParseError),
}
