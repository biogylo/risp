use std::fmt::{Debug, Display, Formatter};
use std::str;
use str::from_utf8;

use crate::parse_error::ParseError;
use crate::parse_error::ParseError::{CannotParseEmpty, CannotParseNumber, MissingDoubleQuote, MissingLeftParenthesis, MissingRightParenthesis, StringDidntEnd};
use crate::tokenize::AstNode::{List, Num, Str, Sym};
use crate::tokenize::AstToken::{Parsed, ParsedRest};

#[derive(Debug, Eq, PartialEq)]
pub enum AstToken<'a> {
    Parsed(AstNode),
    ParsedRest((AstNode, &'a [u8])),
}

#[derive(Eq, PartialEq)]
pub enum AstNode {
    List(Box<[AstNode]>),
    Num(isize),
    Sym(Box<[u8]>),
    Str(Box<[u8]>),
}


impl Display for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            List(nodes) => {
                write!(f, "(", )?;
                let mut node_iter = nodes.iter();
                if let Some(first_node) = node_iter.next() {
                    write!(f, "{}", first_node)?;
                }
                node_iter.map(|node| write!(f, " {}", node)).collect::<Result<_, _>>()?;
                write!(f, ")", )?;
                Ok(())
            }
            Num(number) => {
                write!(f, "{}", number)?;
                Ok(())
            }
            Sym(symbol_buffer) => {
                write!(f, "{}", from_utf8(symbol_buffer).expect("Symbols should always be UTF-8"))?;
                Ok(())
            }
            Str(string_buffer) => {
                write!(f, "\"{}\"", from_utf8(string_buffer).expect("Strings should always be UTF-8"))?;
                Ok(())
            }
        }
    }
}

impl Debug for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            List(nodes) => {
                write!(f, "List(", )?;
                let mut node_iter = nodes.iter();
                if let Some(first_node) = node_iter.next() {
                    write!(f, "{:?}", first_node)?;
                }
                node_iter.map(|node| write!(f, " {:?},", node)).collect::<Result<_, _>>()?;
                write!(f, ")", )?;
                Ok(())
            }
            Num(number) => {
                write!(f, "Num({})", number)?;
                Ok(())
            }
            Sym(symbol_buffer) => {
                write!(f, "Sym({})", from_utf8(symbol_buffer).expect("Symbols should always be UTF-8"))?;
                Ok(())
            }
            Str(string_buffer) => {
                write!(f, "Str({})", from_utf8(string_buffer).expect("Strings should always be UTF-8"))?;
                Ok(())
            }
        }
    }
}

impl AstNode {
    fn _new() -> AstNode {
        vec![].into()
    }

    fn try_parse_atom(buffer: &[u8]) -> Result<AstNode, ParseError> {
        if let Some(bad_char) = buffer.iter().filter(is_atom_forbidden_char).next() {
            return Err(ParseError::ForbiddenCharInSymbol((*bad_char).into()));
        }
        let first_char = buffer
            .get(0)
            .expect("We can't pass an empty atom");

        let atom_is_number = first_char.is_ascii_digit() || *first_char == b'-';
        if !atom_is_number { // Then it is a symbol
            return Ok(Sym(buffer.into()));
        }
        let buffer = from_utf8(buffer).expect("Has to be UTF-8");
        if let Ok(number) = buffer.parse() {
            Ok(Num(number))
        } else {
            Err(CannotParseNumber(buffer.to_string()))
        }
    }

    fn from_symbol(buffer: &[u8]) -> AstNode {
        AstNode::try_parse_atom(buffer).expect("This function is just for unit testing!")
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


const SYMBOL_FORBIDDEN_CHARS: &[u8] = b"()\"\'";


fn is_atom_forbidden_char(c: &&u8) -> bool {
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

fn tokenize_atom(buffer: &[u8]) -> Result<AstToken, ParseError> {
    let trimmed = buffer.trim_ascii();
    let cutting_index = get_cutting_index_for_symbol(trimmed);
    let (to_parse, rest) = trimmed.split_at(cutting_index);
    let trimmed_rest = rest.trim_ascii();
    let node = AstNode::try_parse_atom(to_parse)?;
    if trimmed_rest.is_empty() {
        Ok(Parsed(node))
    } else {
        Ok(ParsedRest((node, rest)))
    }
}

fn tokenize_string(buffer: &[u8]) -> Result<AstToken, ParseError> {
    // Read until end quote
    let Some((full_string, rest)) = buffer.split_once(|c| *c == b'"') else {
        return Err(MissingDoubleQuote);
    };
    let node = Str(full_string.into());
    if rest.is_empty() {
        return Ok(Parsed(node));
    }
    if rest[0].is_ascii_whitespace() {
        return Ok(ParsedRest((node, rest)));
    };
    let rest = rest.trim_ascii_start();
    let rest_first_char = rest.get(0).expect("We know it was not whitespace from before");
    if *rest_first_char == b')' {
        return Ok(ParsedRest((node, rest)));
    } else {
        return Err(StringDidntEnd);
    }
}

// Assuming the token is a list without outer parens -> "x y (y z s) s (f (f)) (s (s ( )))"
// Attempt to return token and rest -> "x", "y (y z s) s (f (f)) (s (s ( )))"
pub fn tokenize(buffer: &[u8]) -> Result<AstToken, ParseError> {
    let trimmed = buffer.trim_ascii();
    let Some((first_char, rest)) = trimmed.split_first() else {
        return Err(CannotParseEmpty);
    };
    if *first_char == b')' {
        return Err(MissingLeftParenthesis);
    };
    if *first_char == b'"' {
        return tokenize_string(rest);
    }
    if first_char != &b'(' {
        // Thank god! we can tokenize this right away!
        return tokenize_atom(trimmed);
    };
    // Pain in the butt! Recursively tokenize -> skip left paren
    let mut trimmed_rest = rest.trim_ascii();
    if trimmed_rest.is_empty() {
        return Err(MissingRightParenthesis);
    };

    let mut nodes = vec![];
    loop {
        let Some((first_char, after_first_char)) = trimmed_rest.split_first() else {
            // Cant be fully parsed since we expect a closing parenthesis
            return Err(MissingRightParenthesis);
        };
        if *first_char == b')' {
            if after_first_char.is_empty() {
                // Nice, we finished
                return Ok(Parsed(nodes.into()));
            } else {
                return Ok(ParsedRest((nodes.into(), after_first_char.trim_ascii())));
            }
        };
        if let ParsedRest((node, rest)) = tokenize(trimmed_rest)? {
            nodes.push(node);
            // No closing paren for us, therefore we must parse another symbol (loop again)
            trimmed_rest = rest.trim_ascii();
        };
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
        assert_matches!(result, Parsed(Num(-5124)));
    }

    #[test]
    fn returns_error_when_empty() {
        let result = tokenize(b"");
        assert_matches!(result, Err(CannotParseEmpty));
    }

    #[test]
    fn returns_error_when_starts_in_endparen() {
        let result = tokenize(b" )ads ");
        assert_matches!(result, Err(MissingLeftParenthesis));
    }


    #[test]
    fn returns_error_when_starts_in_startparen() {
        let result = tokenize(b" (");
        assert_matches!(result, Err(ParseError::MissingRightParenthesis));
    }

    #[test]
    fn returns_empty_list_when_empty_list() {
        let result = tokenize(b"()").unwrap();
        assert_matches!(result, Parsed(List(the_vec)) if the_vec.len() == 0);
    }

    #[test]
    fn returns_string_when_string() {
        let result = tokenize(b"\"asda asdas dasd\"").unwrap();
        assert_matches!(result, Parsed(Str(the_str)) if *the_str == *b"asda asdas dasd");
    }

    #[test]
    fn char_after_quote_in_string_is_bad() {
        let result = tokenize(b"\"asda asdas dasd\"asd");
        assert_matches!(result, Err(StringDidntEnd));
    }


    #[test]
    fn char_after_quote_in_string_is_bad_in_sexpr() {
        let result = tokenize(b"( asd \"asda asdas dasd\"asd )");
        assert_matches!(result, Err(StringDidntEnd));
    }

    #[test]
    fn returns_symbol_trimmed_when_list_has_single_element() {
        let result = tokenize(b"  x    ").unwrap();
        assert_matches!(result, Parsed(Sym(symbol_str)) if symbol_str.as_ref() == b"x");
    }

    #[test]
    fn returns_symbol_trimmed_and_rest_when_starts_with_symbol() {
        let result = tokenize(b"  x   y ").unwrap();
        assert_matches!(
            result,
            AstToken::ParsedRest(
                (Sym(symbol_str), rest_str)
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
                (Sym(symbol_str), rest_str)
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
                (Sym(symbol_str), rest_str)
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
                (Sym(symbol_str), rest_str)
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
                Err(ParseError::ForbiddenCharInSymbol(found)) if found == forbidden_char_ascii.to_char()
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
        let result = tokenize(b"(  +  a (\t* \tb (  +ASDNASC  c    d e) f)   (    *   ( *   \ng h i) j  k) )").unwrap();
        let expected: AstNode = vec![
            b"+".into(),
            b"a".into(),
            vec![
                b"*".into(),
                b"b".into(),
                vec![
                    b"+ASDNASC".into(),
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
        let result = tokenize(b"(  +  abbas (\t* \tadd=addas (  ASDNASC  lakakas    zo*poplapapas donkozupipas&3f) domperign4o3n2)   (    *   ( *   \nswag_swag_swag_1999 blogger i) j  k) )").unwrap();
        let expected: AstNode = vec![
            b"+".into(),
            b"abbas".into(),
            vec![
                b"*".into(),
                b"add=addas".into(),
                vec![
                    b"ASDNASC".into(),
                    b"lakakas".into(),
                    b"zo*poplapapas".into(),
                    b"donkozupipas&3f".into(),
                ].into(),
                b"domperign4o3n2".into(),
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

    #[test]
    fn list_errors_when_an_element_starts_with_digit_but_is_nan() {
        let result = tokenize(b"(  +  abbas (\t* \tadd=addas (  ASDNASC  lakakas    zo*poplapapas donkozupipas&3f) 1domperign4o3n2)   (    *   ( *   \nswag_swag_swag_1999 blogger i) j  k) )");
        assert_matches!(
            result,
            Err(CannotParseNumber(string)) if string == "1domperign4o3n2"
        );
    }
}