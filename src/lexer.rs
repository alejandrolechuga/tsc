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

impl<'input> Lexer<'input> {
    fn new(stream: Chars<'input>) -> Lexer<'input> {
        Lexer {
            column: 1,
            line: 1,
            stream: stream.peekable()
        }
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
            _ => None
        }
    }
}

#[test]
fn does_not_blow_up() {
    Lexer::new("test".chars());
}
