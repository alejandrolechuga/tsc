use std::iter::Peekable;

mod location;
mod token;

use self::location::Location;
use self::token::{CommentStyle,QuoteStyle,Token,TokenType};

struct Lexer<I>
where I: Iterator<Item = char>,
{
    column: u32,
    line: u32,
    stream: Peekable<I>,
}

#[derive(Debug)]
enum LexError {
    UnexpectedEndOfInput(Location),
    UnexpectedCharacter(Location),
}

impl<I> Lexer<I>
where I: Iterator<Item = char>,
{
    fn new(stream: I) -> Lexer<I> {
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

    fn scalar(&mut self, loc: Location, typ: TokenType) -> Token {
        self.skip();
        Token::new(loc, typ)
    }

    fn comment(&mut self, loc: Location, style: CommentStyle) -> Result<Token, LexError> {
        let mut s = String::new();
        self.skip();
        match style {
            CommentStyle::SingleLine => {
                loop {
                    match self.next_char() {
                        Some(c) if is_line_terminator(c) => break,
                        Some(c) => s.push(c),
                        None => break,
                    }
                }
            },
            CommentStyle::MultiLine => {
                loop {
                    match self.next_char() {
                        Some('*') => {
                            match self.next_char() {
                                Some('/') => break,
                                Some(c) => {
                                    s.push('*');
                                    s.push(c);
                                },
                                None => return Err(LexError::UnexpectedEndOfInput(loc)),
                            }
                        },
                        Some(c) => s.push(c),
                        None => return Err(LexError::UnexpectedEndOfInput(loc)),
                    }
                }
            },
        }
        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::Comment(s, style)))
    }

    fn ws(&mut self, loc: Location) -> Token {
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

    fn string(&mut self, loc: Location, quote: QuoteStyle) -> Result<Token, LexError> {
        let mut s = String::new();

        self.skip();

        loop {
            match self.next_char() {
                Some('"') if quote == QuoteStyle::Double => break,
                Some('\'') if quote == QuoteStyle::Single => {
                    break
                },
                Some('\\') => {
                    match self.escape_or_line_continuation() {
                        Ok(escape_or_continuation) => s.push_str(&escape_or_continuation),
                        Err(x) => return Err(x),
                    }
                },
                Some(c) if is_line_terminator(c) => return Err(LexError::UnexpectedCharacter(self.get_location())),
                Some(c) => {
                    s.push(c)
                },
                None => return Err(LexError::UnexpectedEndOfInput(self.get_location())),
            }
        }

        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::String(s, quote)))
    }

    fn comment_or_div(&mut self, loc: Location) -> Result<Token, LexError> {
        self.skip();
        match self.peek() {
            Some('/') => self.comment(loc, CommentStyle::SingleLine),
            Some('*') => self.comment(loc, CommentStyle::MultiLine),
            Some('=') => Ok(self.scalar(loc, TokenType::DivEqual)),
            None |
            Some(_)   => Ok(self.scalar(loc, TokenType::Div)),
        }
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

    fn equal(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('>') => {
                self.next_char();
                Token::new(loc, TokenType::Arrow)
            },
            Some('=') => {
                self.next_char();
                match self.peek() {
                    Some('=') => {
                        self.next_char();
                        Token::new(loc, TokenType::TripleEquals)
                    },
                    _ => Token::new(loc, TokenType::DoubleEquals)
                }
            },
            Some(_) |
            None => Token::new(loc, TokenType::Equals)
        }
    }

    fn bang(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('=') => {
                self.next_char();
                match self.peek() {
                    Some('=') => {
                        self.next_char();
                        Token::new(loc, TokenType::NotTripleEquals)
                    },
                    _ => Token::new(loc, TokenType::NotEquals)
                }
            },
            _ => Token::new(loc, TokenType::Bang),
        }
    }

    fn ampersand(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('&') => {
                self.next_char();
                Token::new(loc, TokenType::LogicalAnd)
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::BinaryAndEquals)
            },
            _ => Token::new(loc, TokenType::BinaryAnd),
        }
    }

    fn pipe(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('|') => {
                self.next_char();
                Token::new(loc, TokenType::LogicalOr)
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::BinaryOrEquals)
            },
            _ => Token::new(loc, TokenType::BinaryOr),
        }
    }

    fn caret(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::BinaryXorEquals)
            },
            _ => Token::new(loc, TokenType::BinaryXor),
        }
    }

    fn minus(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('-') => {
                self.next_char();
                Token::new(loc, TokenType::Decrement)
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::MinusEquals)
            },
            _ => Token::new(loc, TokenType::Minus),
        }
    }

    fn plus(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('+') => {
                self.next_char();
                Token::new(loc, TokenType::Increment)
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::PlusEquals)
            },
            _ => Token::new(loc, TokenType::Plus),
        }
    }

    fn gt(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::GreaterThanEqualTo)
            },
            Some('>') => {
                self.next_char();
                match self.peek() {
                    Some('=') => {
                        self.skip();
                        Token::new(loc, TokenType::RightShiftEquals)
                    },
                    Some('>') => {
                        self.skip();
                        match self.peek() {
                            Some('=') => {
                                self.skip();
                                Token::new(loc, TokenType::TripleRightShiftEquals)
                            },
                            _ => Token::new(loc, TokenType::TripleRightShift)
                        }
                    },
                    _ => Token::new(loc, TokenType::RightShift)
                }
            },
            _ => Token::new(loc, TokenType::GreaterThan)
        }
    }

    fn lt(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('<') => {
                self.next_char();
                match self.peek() {
                    Some('=') => {
                        self.next_char();
                        Token::new(loc, TokenType::LeftShiftEquals)
                    },
                    _ => Token::new(loc, TokenType::LeftShift)
                }
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::LessThanEqualTo)
            },
            _ => Token::new(loc, TokenType::LessThan)
        }
    }

    fn period(&mut self, loc: Location) -> Result<Token, LexError> {
        self.skip();
        match self.peek() {
            Some('.') => {
                self.next_char();
                match self.next_char() {
                    Some('.') => Ok(Token::new(loc, TokenType::Ellipsis)),
                    Some(_)   => Err(LexError::UnexpectedCharacter(loc)),
                    None      => Err(LexError::UnexpectedEndOfInput(loc)),
                }
            },
            _ => Ok(Token::new(loc, TokenType::Period))
        }
    }

    fn percent(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::PercentEquals)
            },
            _ => Token::new(loc, TokenType::Percent)
        }
    }

    fn asterisk(&mut self, loc: Location) -> Token {
        self.skip();
        match self.peek() {
            Some('*') => {
                self.next_char();
                match self.peek() {
                    Some('=') => {
                        self.next_char();
                        Token::new(loc, TokenType::PowerEquals)
                    },
                    _ => Token::new(loc, TokenType::Power)
                }
            },
            Some('=') => {
                self.next_char();
                Token::new(loc, TokenType::TimesEquals)
            },
            _ => Token::new(loc, TokenType::Times)
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

impl<I> Iterator for Lexer<I>
where I: Iterator<Item = char>
{
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Result<Token, LexError>> {
        let loc = self.get_location();
        self.peek().map(|next| {
            match next {
                x if is_ws(x) => Ok(self.ws(loc)),
                '/'  => self.comment_or_div(loc),
                '\'' => self.string(loc, QuoteStyle::Single),
                '"'  => self.string(loc, QuoteStyle::Double),
                '{'  => Ok(self.scalar(loc, TokenType::LeftBrace)),
                '}'  => Ok(self.scalar(loc, TokenType::RightBrace)),
                '='  => Ok(self.equal(loc)),
                '!'  => Ok(self.bang(loc)),
                '&'  => Ok(self.ampersand(loc)),
                '|'  => Ok(self.pipe(loc)),
                '^'  => Ok(self.caret(loc)),
                '('  => Ok(self.scalar(loc, TokenType::LeftParen)),
                ')'  => Ok(self.scalar(loc, TokenType::RightParen)),
                ']'  => Ok(self.scalar(loc, TokenType::RightBracket)),
                '['  => Ok(self.scalar(loc, TokenType::LeftBracket)),
                ':'  => Ok(self.scalar(loc, TokenType::Colon)),
                ','  => Ok(self.scalar(loc, TokenType::Comma)),
                '+'  => Ok(self.plus(loc)),
                '-'  => Ok(self.minus(loc)),
                '>'  => Ok(self.gt(loc)),
                '<'  => Ok(self.lt(loc)),
                '.'  => self.period(loc),
                '%'  => Ok(self.percent(loc)),
                '*'  => Ok(self.asterisk(loc)),
                '~'  => Ok(self.scalar(loc, TokenType::Tilde)),
                ';'  => Ok(self.scalar(loc, TokenType::Semicolon)),
                '?'  => Ok(self.scalar(loc, TokenType::Question)),
                _    => Err(LexError::UnexpectedCharacter(loc)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_whitespace() {
        let input = " \t\t ";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_comments() {
        let input = "//this is a comment";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_single_strings() {
        let input = "'this is a string'";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_double_strings() {
        let input = r#""this is a string""#;
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_div() {
        let input = "/";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_div_equals() {
        let input = "/=";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    #[test]
    fn identifies_right_brace() {
        let input = "}";
        let output = first_token(input);
        assert_eq!(token_text(output), input);
    }

    fn first_token(input: &str) -> Option<Result<Token, LexError>> {
        let mut lexer = Lexer::new(input.chars());
        lexer.next()
    }

    fn token_text(tok: Option<Result<Token, LexError>>) -> String {
        match tok {
            Some(Ok(t)) => t.typ.to_string(),
            Some(Err(e)) => panic!("{:?}", e),
            None => panic!("Didn't get a token from the lexer")
        }
    }
}
