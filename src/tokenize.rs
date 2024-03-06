use std::ffi::FromBytesWithNulError;
use thiserror::Error;
use crate::parse_error::{ParseError, SymbolParseError};
use crate::tokenize::AstNode::List;


#[derive(Debug, Eq, PartialEq)]
enum AstToken<'a> {
    Parsed(AstNode),
    ParsedRest((AstNode, &'a [u8])),
}

#[derive(Debug, Eq, PartialEq)]
enum AstNode {
    List(Vec<AstNode>),
    // Int(isize),
    Symbol(Symbol),
}

#[derive(Debug, Eq, PartialEq)]
struct Symbol(Box<[u8]>);


const FORBIDDEN_CHARS: &[u8; 6] = b"(){}<>";

impl Symbol {
    fn is_forbidden_char(c: &u8) -> bool {
        FORBIDDEN_CHARS.contains(c)
    }
}

impl TryFrom<&[u8]> for Symbol {
    type Error = SymbolParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(bad_char) = value.iter().filter(|&c| Symbol::is_forbidden_char(c)).next() {
            return Err(Self::Error::ForbiddenCharInSymbol(*bad_char));
        }
        let boxed_value: Box<[u8]> = value.into();
        Ok(
            Symbol {
                0: boxed_value
            }
        )
    }
}

fn tokenize_symbol(buffer: &[u8]) -> Result<AstToken, SymbolParseError> {
    let to_parse = buffer.trim_ascii();
    match to_parse.split_once(u8::is_ascii_whitespace) {
        None => {
            let symbol = to_parse.try_into()?;
            let node = AstNode::Symbol(symbol);
            let token = AstToken::Parsed(node);
            Ok(token)
        }
        Some((to_parse, rest)) => {
            let symbol = to_parse.try_into()?;
            let node = AstNode::Symbol(symbol);
            let token = AstToken::ParsedRest((node, rest));
            Ok(token)
        }
    }
}

// Assuming the token is a list without outer parens -> "x y (y z s) s (f (f)) (s (s ( )))"
// Attempt to return token and rest -> "x", "y (y z s) s (f (f)) (s (s ( )))"
fn tokenize(buffer: &[u8]) -> Result<AstToken, ParseError> {
    let trimmed = buffer.trim_ascii();
    let Some(first_char) = trimmed.first() else {
        // Nothing left to tokenize
        let inner_node = AstNode::List(vec![]);
        let token = AstToken::Parsed(inner_node);
        return Ok(token);
    };
    if first_char != &b'(' {
        // Thank god! we can tokenize this right away!
        return tokenize_symbol(trimmed).map_err(|e| e.into());
    }
    todo!()
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;
    use crate::tokenize::AstToken::Parsed;
    use super::*;

    #[test]
    fn returns_empty_node_when_empty() {
        let result = tokenize(b"").unwrap();
        assert_matches!(result, Parsed(AstNode::List(the_vec)) if the_vec.len() == 0);
    }

    #[test]
    fn returns_symbol_trimmed_when_list_has_single_element() {
        let result = tokenize(b"  x    ").unwrap();
        assert_matches!(result, Parsed(AstNode::Symbol(Symbol(symbol_str))) if symbol_str.as_ref() == b"x");
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol() {
        let result = tokenize(b"  x   y ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (AstNode::Symbol(Symbol(symbol_str)), rest_str)
            ) if symbol_str.as_ref() == b"x"
                    && rest_str.trim_ascii() == b"y"
        );
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol_complex() {
        let result = tokenize(b"  x   y z (x t ) d ( (d) ) ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (AstNode::Symbol(Symbol(symbol_str)), rest_str)
            ) if symbol_str.as_ref() == b"x"
                    && rest_str.trim_ascii() == b"y z (x t ) d ( (d) )"
        );
    }
}