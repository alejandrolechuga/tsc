use std::iter::Peekable;
use std::str::Chars;

enum TokenType {
    Null,
    String(String),
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

enum LexError {
    UnexpectedEndOfInput(Location),
    UnexpectedCharacter(Location),
}

struct Location {
    column: u32,
    line: u32,
}

impl Location {
    fn new(line: u32, column: u32) -> Location {
        Location { line, column }
    }
}

impl<'input> Lexer<'input> {
    fn new(stream: Chars<'input>) -> Lexer<'input> {
        Lexer {
            column: 1,
            line: 1,
            stream: stream.peekable()
        }
    }

    fn get_location(self) -> Location {
        Location::new(self.line, self.column)
    }

    fn peek(self) -> Option<&'input char> {
        self.stream.peek()
    }

    fn escape_sequence(self) -> Result<String, LexError> {
        Err(LexError::UnexpectedEndOfInput(self.get_location()))
    }

    fn lex_string(&mut self, delimeter: char) -> Result<Token, LexError> {
        let mut s = String::new();
        s.push(delimeter);

        loop {
            let next = self.next_char();
            match next {
                Some(c) => {
                    s.push(c);
                    if c == delimeter {
                        break;
                    }
                    if c == '\\' {
                        match self.peek() {
                            // Escape sequence
                            Some(_n) => {
                                let mut seq = self.escape_sequence();
                                match seq {
                                    Ok(x) => s.push_str(&x),
                                    Err(x) => return Err(x)
                                }
                            }
                            // Line continuation
                            // EOF
                            None => {
                                return Err(LexError::UnexpectedEndOfInput(self.get_location()))
                            }
                        }
                    }
                },
                None => {
                    return Err(LexError::UnexpectedEndOfInput(self.get_location()))
                }
            }

        }
        s.shrink_to_fit();
        Ok(Token {
            line: self.line,
            column: self.column,
            typ: TokenType::String(s)
        })
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
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Result<Token, LexError>> {
        match self.next_char() {
            Some('n') => {
                Some(Ok(Token::new(self.line, self.column, TokenType::Null)))
            },
            Some('\'') => Some(self.lex_string('\'')),
            Some('"') => Some(self.lex_string('"')),
            _ => None
        }
    }
}

#[test]
fn does_not_blow_up() {
    Lexer::new("test".chars());
}
