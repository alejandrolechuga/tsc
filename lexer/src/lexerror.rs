use super::location::Location;

#[derive(Debug)]
pub enum LexError {
    InvalidCodePoint(String),
    UnexpectedEndOfInput(Location),
    UnexpectedCharacter(Location, char),
}
