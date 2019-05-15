use std::fmt;
use super::location::Location;

#[derive(PartialEq)]
pub enum QuoteStyle {
    Single,
    Double,
}

impl fmt::Display for QuoteStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QuoteStyle::Single => write!(f, "'"),
            QuoteStyle::Double => write!(f, "\"")
        }
    }
}

pub enum CommentStyle {
    SingleLine,
    MultiLine,
}

pub struct Token {
    pub column: u32,
    pub line: u32,
    pub typ: TokenType,
}

pub enum TokenType {
    Comment(String, CommentStyle),
    Div,
    DivEqual,
    RightBrace,
    String(String, QuoteStyle),
    WhiteSpace(String),
}

impl Token {
    pub fn new(loc: Location, typ: TokenType) -> Token {
        Token {
            column: loc.column,
            line: loc.line,
            typ,
        }
    }
}
