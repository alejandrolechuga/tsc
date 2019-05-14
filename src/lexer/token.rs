use super::location::Location;

#[derive(PartialEq)]
pub enum QuoteStyle {
    Single,
    Double,
}

pub struct Token {
    pub column: u32,
    pub line: u32,
    pub typ: TokenType,
}

pub enum TokenType {
    Comment(String),
    WhiteSpace(String),
    String(String, QuoteStyle),
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
