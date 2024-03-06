use std::fmt::{Debug, Formatter};
use std::str;
use str::from_utf8;

use crate::parse_error::ParseError;
use crate::tokenize::AstNode::{List, Number, Symbol};
use crate::tokenize::AstToken::{Parsed, ParsedRest};

#[derive(Debug, Eq, PartialEq)]
pub enum AstToken<'a> {
    Parsed(AstNode),
    ParsedRest((AstNode, &'a [u8])),
}

#[derive(Eq, PartialEq)]
pub enum AstNode {
    List(Box<[AstNode]>),
    Number(isize),
    Symbol(Box<[u8]>),
}

impl Debug for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            List(nodes) => {
                write!(f, "[", )?;
                nodes.iter().map(|node| write!(f, "{:?}, ", node)).collect::<Result<_, _>>()?;
                write!(f, "]", )?;
                Ok(())
            }
            Number(number) => {
                write!(f, "{}", number)?;
                Ok(())
            }
            Symbol(symbol_buffer) => {
                write!(f, "\"{}\"", from_utf8(symbol_buffer).expect("Symbols should always be UTF-8"))?;
                Ok(())
            }
        }
    }
}

impl AstNode {
    fn _new() -> AstNode {
        vec![].into()
    }

    fn try_parse_symbol(buffer: &[u8]) -> Result<AstNode, ParseError> {
        if let Some(bad_char) = buffer.iter().filter(is_symbol_forbidden_char).next() {
            return Err(ParseError::ForbiddenCharInSymbol((*bad_char).into()));
        }
        if let Ok(number) = from_utf8(buffer).expect("We shouldn't be taking weird strings").parse() {
            Ok(Number(number))
        } else {
            Ok(Symbol(buffer.into()))
        }
    }

    fn from_symbol(buffer: &[u8]) -> AstNode {
        AstNode::try_parse_symbol(buffer).expect("This function is just for unit testing!")
    }
}

impl From<Vec<AstNode>> for AstNode {
    fn from(value: Vec<AstNode>) -> Self {
        List(value.into_boxed_slice())
    }
}

impl<const N: usize> From<&[u8; N]> for AstNode {
    fn from(value: &[u8; N]) -> Self {
        Self::from_symbol(value)
    }
}


const SYMBOL_FORBIDDEN_CHARS: &[u8] = b"[](){}<>\"\'";


fn is_symbol_forbidden_char(c: &&u8) -> bool {
    SYMBOL_FORBIDDEN_CHARS.contains(*c)
}


fn get_cutting_index_for_symbol(trimmed_symbol_buffer: &[u8]) -> usize {
    let first_closing_paren_idx = trimmed_symbol_buffer.iter().position(|&c| c == b')');
    let first_whitespace_idx = trimmed_symbol_buffer.iter().position(u8::is_ascii_whitespace);
    match (first_closing_paren_idx, first_whitespace_idx) {
        (Some(pindx), Some(windx)) => {
            // If the whitespace is right after the parens, remove parens
            if windx == pindx + 1 {
                pindx
            } else {
                windx
            }
        }
        (None, Some(windx)) => {
            windx
        }
        (Some(pinx), None) => {
            pinx
        }
        (None, None) => { trimmed_symbol_buffer.len() }
    }
}

fn tokenize_symbol(buffer: &[u8]) -> Result<AstToken, ParseError> {
    let trimmed = buffer.trim_ascii();
    let cutting_index = get_cutting_index_for_symbol(trimmed);
    let (to_parse, rest) = trimmed.split_at(cutting_index);
    let trimmed_rest = rest.trim_ascii();
    let node = AstNode::try_parse_symbol(to_parse)?;
    if trimmed_rest.is_empty() {
        Ok(Parsed(node))
    } else {
        Ok(ParsedRest((node, rest)))
    }
}

// Assuming the token is a list without outer parens -> "x y (y z s) s (f (f)) (s (s ( )))"
// Attempt to return token and rest -> "x", "y (y z s) s (f (f)) (s (s ( )))"
pub fn tokenize(buffer: &[u8]) -> Result<AstToken, ParseError> {
    let trimmed = buffer.trim_ascii();
    let Some(first_char) = trimmed.first() else {
        // Nothing left to tokenize
        let inner_node = AstNode::List(vec![].into());
        let token = AstToken::Parsed(inner_node);
        return Ok(token);
    };
    if first_char != &b'(' {
        // Thank god! we can tokenize this right away!
        return tokenize_symbol(trimmed).map_err(|e| e.into());
    };
    let mut nodes = vec![];
    // Pain in the butt! Recursively tokenize (skip left paren
    let mut to_parse = &buffer[1..];
    loop {
        let ParsedRest((node, rest)) = tokenize(to_parse)? else {
            // Cant be fully parsed since we expect a closing parenthesis
            return Err(ParseError::MissingRightParenthesis);
        };
        nodes.push(node);
        let trimmed_rest = rest.trim_ascii();
        if trimmed_rest[0] == b')' {
            if trimmed_rest.len() == 1 {
                // Nice, we finished
                return Ok(Parsed(nodes.into()));
            } else {
                return Ok(ParsedRest((nodes.into(), &trimmed_rest[1..])));
            }
        }
        // No closing paren for us, therefore we must parse another symbol (loop again)
        to_parse = trimmed_rest;
    };
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::tokenize::AstToken::Parsed;

    use super::*;

    #[test]
    fn number_tokenized() {
        let result = tokenize(b"  -5124  ").unwrap();
        assert_matches!(result, Parsed(Number(-5124)));
    }

    #[test]
    fn returns_empty_node_when_empty() {
        let result = tokenize(b"").unwrap();
        assert_matches!(result, Parsed(AstNode::List(the_vec)) if the_vec.len() == 0);
    }

    #[test]
    fn returns_symbol_trimmed_when_list_has_single_element() {
        let result = tokenize(b"  x    ").unwrap();
        assert_matches!(result, Parsed(Symbol(symbol_str)) if symbol_str.as_ref() == b"x");
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol() {
        let result = tokenize(b"  x   y ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (Symbol(symbol_str), rest_str)
            ) if symbol_str.as_ref() == b"x"
                    && rest_str.trim_ascii() == b"y"
        );
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol_complex() {
        let result = tokenize(b"  xasd   y z (x t ) d ( (d) ) ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (Symbol(symbol_str), rest_str)
            ) if symbol_str.as_ref() == b"xasd"
                    && rest_str.trim_ascii() == b"y z (x t ) d ( (d) )"
        );
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol_complex_symb() {
        let result = tokenize(b"  +   y z (x t ) d ( (d) ) ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (Symbol(symbol_str), rest_str)
            ) if symbol_str.as_ref() == b"+"
                    && rest_str.trim_ascii() == b"y z (x t ) d ( (d) )"
        );
    }

    #[test]
    fn returns_symbol_trimmed_up_til_before_parenthesis_and_rest() {
        let result = tokenize(b"  +)   y z (x t ) d ( (d) ) ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (Symbol(symbol_str), rest_str)
            ) if symbol_str.as_ref() == b"+"
                    && rest_str.trim_ascii() == b")   y z (x t ) d ( (d) )"
        );
    }


    #[test]
    fn returns_parse_error_when_forbidden_char() {
        for forbidden_char in SYMBOL_FORBIDDEN_CHARS {
            let forbidden_char_ascii = forbidden_char.as_ascii().unwrap();
            let input = format!("  x{}z   y z (x t ) d ( (d) ) ", forbidden_char_ascii);
            println!("{input}");
            let result = tokenize(input.as_bytes());

            assert_matches!(
                result,
                Err(ParseError::ForbiddenCharInSymbol(found.into())) if found == *forbidden_char
            );
        }
    }


    #[test]
    fn list_gets_parsed_correctly() {
        let result = tokenize(b"(  a  b (x y) z)").unwrap();
        let expected: AstNode = vec![
            b"a".into(),
            b"b".into(),
            vec![
                b"x".into(),
                b"y".into(),
            ].into(),
            b"z".into(),
        ].into();
        assert_matches!(
            result,
            Parsed(ast_node) if ast_node == expected);
    }


    #[test]
    fn list_gets_parsed_correctly_complex() {
        let result = tokenize(b"(  +  a (* b c d) e )").unwrap();
        let expected: AstNode = vec![
            b"+".into(),
            b"a".into(),
            vec![
                b"*".into(),
                b"b".into(),
                b"c".into(),
                b"d".into(),
            ].into(),
            b"e".into(),
        ].into();
        assert_matches!(
            result,
            Parsed(ast_node) if ast_node == expected);
    }


    #[test]
    fn list_gets_parsed_correctly_complex_2() {
        let result = tokenize(b"(  +  a (\t* \tb (  -  c    d e) f)   (    *   ( *   \ng h i) j  k) )").unwrap();
        let expected: AstNode = vec![
            b"+".into(),
            b"a".into(),
            vec![
                b"*".into(),
                b"b".into(),
                vec![
                    b"-".into(),
                    b"c".into(),
                    b"d".into(),
                    b"e".into(),
                ].into(),
                b"f".into(),
            ].into(),
            vec![
                b"*".into(),
                vec![
                    b"*".into(),
                    b"g".into(),
                    b"h".into(),
                    b"i".into(),
                ].into(),
                b"j".into(),
                b"k".into(),
            ].into(),
        ].into();
        assert_matches!(
            result,
            Parsed(ast_node) if ast_node == expected);
    }

    #[test]
    fn list_gets_parsed_correctly_complex_3() {
        let result = tokenize(b"(  +  abbas (\t* \tadd=addas (  -  lakakas    zo*poplapapas donkozupipas&3f) 1domperign4o3n2)   (    *   ( *   \nswag_swag_swag_1999 blogger i) j  k) )").unwrap();
        let expected: AstNode = vec![
            b"+".into(),
            b"abbas".into(),
            vec![
                b"*".into(),
                b"add=addas".into(),
                vec![
                    b"-".into(),
                    b"lakakas".into(),
                    b"zo*poplapapas".into(),
                    b"donkozupipas&3f".into(),
                ].into(),
                b"1domperign4o3n2".into(),
            ].into(),
            vec![
                b"*".into(),
                vec![
                    b"*".into(),
                    b"swag_swag_swag_1999".into(),
                    b"blogger".into(),
                    b"i".into(),
                ].into(),
                b"j".into(),
                b"k".into(),
            ].into(),
        ].into();
        assert_matches!(
            result,
            Parsed(ast_node) if ast_node == expected);
    }
}