use std::iter::Peekable;
use std::str::Chars;

enum TokenType {
    Null,
}

struct Token {
    column: u32,
    line: u32,
    typ: TokenType,
}

impl Token {
    fn new(line: u32, column: u32, typ: TokenType) -> Token {
        Token { column, line, typ }
    }
}

struct Lexer<'input> {
    column: u32,
    line: u32,
    stream: Peekable<Chars<'input>>,
}

struct ParseError {
    column: u32,
    line: u32,
}

impl<'input> Lexer<'input> {
    fn new(stream: Chars<'input>) -> Lexer<'input> {
        Lexer {
            column: 1,
            line: 1,
            stream: stream.peekable()
        }
    }

    fn lex_string(&mut self, delimeter: char) -> Result<String, ParseError> {
        let mut s = String::new();
        s.push(delimeter);
        loop {
            match self.next_char() {
                Some(c) => {
                    s.push(c);
                    if c == delimeter
                        break;
                    if c == '\\':
                        // Escape sequence
                        // Line continuation
                },
                None => Error(ParseError { line: self.line, column: self.column })
            }

        }
        s.shrink_to_fit();
        s
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
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.next_char() {
            Some('n') => {
                Some(Token::new(self.line, self.column, TokenType::Null))
            },
            Some('\'') => self.lex_string('\''),
            Some('"') => self.lex_string('"'),
            _ => None
        }
    }
}

#[test]
fn does_not_blow_up() {
    Lexer::new("test".chars());
}
