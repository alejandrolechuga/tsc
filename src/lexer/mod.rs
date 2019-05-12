use regex::Regex;
use std::iter::Peekable;
use std::str::Chars;

mod location;
mod token;

use self::location::Location;
use self::token::{Token, TokenType};

struct Lexer<'input> {
    column: u32,
    line: u32,
    stream: Peekable<Chars<'input>>,
}

enum LexError {
    UnexpectedEndOfInput(Location),
    UnexpectedCharacter(Location),
}

impl<'input> Lexer<'input> {
    fn new(stream: Chars<'input>) -> Lexer<'input> {
        Lexer {
            column: 1,
            line: 1,
            stream: stream.peekable()
        }
    }

    fn get_location(&self) -> Location {
        Location::new(self.line, self.column)
    }

    fn next_char(&mut self) -> Option<char> {
        match self.stream.next() {
            Some('\n') => {
                self.line += 1;
                self.column = 1;
                Some('\n')
            },
            Some(x) => {
                self.column += 1;
                Some(x)
            },
            None => None,
        }
    }

    fn is_ws(c: &char) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[\u0009\u000B\u000C\u0020\u00A0\uFEFF\p{Space_Separator}]").unwrap();
        }

        RE.is_match(&format!("{}", c))
    }

    fn ws(&mut self) -> Token {
        let l = self.get_location();
        let mut s = String::new();
        loop {
            match self.stream.peek() {
                Some(c) if Lexer::is_ws(c) => s.push(*c),
                _ => break
            }
        }
        Token::new(l, TokenType::WhiteSpace(s))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Result<Token, LexError>> {
        match self.next_char() {
            Some(x) if Lexer::is_ws(&x) => Some(Ok(self.ws())),
            _ => None
        }
    }
}

#[test]
fn does_not_blow_up() {
    Lexer::new("test".chars());
}
