use std::iter::Peekable;
use std::str::Chars;

mod location;
mod token;

use self::location::Location;
use self::token::{QuoteStyle,Token,TokenType};

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

    fn peek(&mut self) -> Option<char> {
        self.stream.peek().map(|x| *x)
    }

    fn skip(&mut self) {
        self.stream.next();
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
        s.shrink_to_fit();
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
        s.shrink_to_fit();
        Token::new(loc, TokenType::WhiteSpace(s))
    }

    fn string(&mut self, quote: QuoteStyle) -> Result<Token, LexError> {
        let loc = self.get_location();
        let mut s = String::new();

        loop {
            match self.next_char() {
                Some('"') if quote == QuoteStyle::Double => break,
                Some('\'') if quote == QuoteStyle::Single => break,
                Some('\\') => {
                    match self.escape_or_line_continuation() {
                        Ok(escape_or_continuation) => s.push_str(&escape_or_continuation),
                        Err(x) => return Err(x),
                    }
                },
                Some(c) if is_line_terminator(c) => return Err(LexError::UnexpectedCharacter(self.get_location())),
                Some(c) => s.push(c),
                None => return Err(LexError::UnexpectedEndOfInput(self.get_location())),
            }
        }

        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::String(s, quote)))
    }

    fn escape_or_line_continuation(&mut self) -> Result<String, LexError> {
        match self.next_char() {
            // line continuation
            Some(c) if is_line_terminator(c) => {
                if c == '\u{000D}' && self.peek() == Some('\u{000A}') {
                    self.skip();
                    let s = "\\\u{000D}\u{000A}".to_string();
                    Ok(s)
                } else {
                    Ok(format!("\\{}", c))
                }
            },
            // Quote
            Some(c) if c == '"' || c == '\'' => Ok(format!("\\{}", c)),
            // Backslash
            Some(c) if c == '\\' => Ok(format!("\\\\")),
            // Something else
            Some(c) => Ok(format!("\\{}", c)),
            None => Err(LexError::UnexpectedEndOfInput(self.get_location())),
        }
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
        self.peek().map(|next| {
            match next {
                x if is_ws(x) => Ok(self.ws()),
                '/'  => Ok(self.comment()),
                '\'' => self.string(QuoteStyle::Single),
                '"'  => self.string(QuoteStyle::Double),
                _ => Err(LexError::UnexpectedCharacter(self.get_location())),
            }
        })
    }
}
