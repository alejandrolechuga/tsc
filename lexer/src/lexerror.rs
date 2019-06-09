#[derive(Debug)]
pub enum LexError {
    InvalidCodePoint(String),
    UnexpectedEndOfInput(u32, u32),
    UnexpectedCharacter(u32, u32, char),
}
