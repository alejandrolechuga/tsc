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

    fn peek(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    fn comment(&mut self) -> Token {
        let loc = self.get_location();
        let mut s = String::new();
        loop {
            match self.next_char() {
                Some(c) if is_line_terminator(c) => break,
                Some(c) => s.push(c),
                None => break,
            }
        }
        Token::new(loc, TokenType::Comment(s))
    }

    fn ws(&mut self) -> Token {
        let loc = self.get_location();
        let mut s = String::new();
        loop {
            match self.next_char() {
                Some(c) if is_ws(c) => s.push(c),
                _ => break
            }
        }
        Token::new(loc, TokenType::WhiteSpace(s))
    }
}


fn is_line_terminator(c: char) -> bool {
    match c {
        '\u{000A}' => true,
        '\u{000D}' => true,
        '\u{2028}' => true,
        '\u{2029}' => true,
        _          => false,
    }
}

fn is_ws(c: char) -> bool {
    match c {
        '\u{0009}' => true,
        '\u{000B}' => true,
        '\u{000C}' => true,
        '\u{0020}' => true,
        '\u{00A0}' => true,
        '\u{1680}' => true,
        '\u{2000}' => true,
        '\u{2001}' => true,
        '\u{2002}' => true,
        '\u{2003}' => true,
        '\u{2004}' => true,
        '\u{2005}' => true,
        '\u{2006}' => true,
        '\u{2007}' => true,
        '\u{2008}' => true,
        '\u{2009}' => true,
        '\u{200A}' => true,
        '\u{202F}' => true,
        '\u{205F}' => true,
        '\u{FEFF}' => true,
        _          => false,
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Result<Token, LexError>> {
        let next = self.peek().map(|x| (*x));
        match next {
            Some(x) if is_ws(x) => Some(Ok(self.ws())),
            Some('/') => Some(Ok(self.comment())),
            None => None,
            _ => Some(Err(LexError::UnexpectedCharacter(self.get_location()))),
        }
    }
}
