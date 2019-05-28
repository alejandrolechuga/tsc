use super::location::Location;

#[derive(Debug)]
pub enum LexError {
    UnexpectedEndOfInput(Location),
    UnexpectedCharacter(Location, char),
}
