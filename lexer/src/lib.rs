extern crate unicode;

use std::iter::Peekable;
use unicode::{is,UnicodeProperty};

mod lexerror;
mod location;
mod token;

use self::location::Location;
use self::token::{CommentStyle,QuoteStyle,Token,TokenType};
use self::lexerror::LexError;

pub struct Lexer<I>
where I: Iterator<Item = char>,
{
    column: u32,
    line: u32,
    stream: Peekable<I>,
}

impl<I> Lexer<I>
where I: Iterator<Item = char>,
{
    pub fn new(stream: I) -> Lexer<I> {
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
        self.next_char();
    }

    fn skip_if(&mut self, c: char) -> bool {
        match self.peek() {
            Some(x) if c == x => {
                self.skip();
                true
            },
            _ => false,
        }
    }

    fn push_while<P>(&mut self, s: &mut String, predicate: P)
    where P: Fn(char) -> bool,
    {
        loop {
            match self.peek() {
                Some(x) => {
                    if !predicate(x) {
                        break
                    }
                    s.push(x);
                    self.skip();
                },
                None => break
            }
        }
    }

    fn unexpected_char(&self, c: char) -> LexError {
        let loc = self.get_location();
        LexError::UnexpectedCharacter(loc, c)
    }

    fn unexpected_eof(&self) -> LexError {
        let loc = self.get_location();
        LexError::UnexpectedEndOfInput(loc)
    }

    fn scalar(&mut self, loc: Location, typ: TokenType) -> Token {
        self.skip();
        Token::new(loc, typ)
    }

    fn comment(&mut self, loc: Location, style: CommentStyle) -> Result<Token, LexError> {
        let mut s = String::new();
        self.skip();
        match style {
            CommentStyle::SingleLine => self.push_while(&mut s, is_line_terminator),
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
                                None => return Err(self.unexpected_eof()),
                            }
                        },
                        Some(c) => s.push(c),
                        None => return Err(self.unexpected_eof()),
                    }
                }
            },
        }
        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::Comment(s, style)))
    }

    fn ws(&mut self) -> TokenType {
        let mut s = String::new();
        self.push_while(&mut s, is_ws);
        s.shrink_to_fit();
        TokenType::WhiteSpace(s)
    }

    fn string(&mut self, quote: QuoteStyle) -> Result<TokenType, LexError> {
        let mut s = String::new();

        // Skip leading quote
        self.skip();

        loop {
            match self.next_char() {
                Some('"')  if quote == QuoteStyle::Double => break,
                Some('\'') if quote == QuoteStyle::Single => break,
                Some('\\') => {
                    match self.escape_or_line_continuation() {
                        Ok(escape) => s.push_str(&escape),
                        Err(x) => return Err(x),
                    }
                },
                Some(c) if is_line_terminator(c) => return Err(self.unexpected_char(c)),
                Some(c) => s.push(c),
                None => return Err(self.unexpected_eof()),
            }
        }

        s.shrink_to_fit();
        Ok(TokenType::String(s, quote))
    }

    // matches constructs beginning with a digit, e.g. 0.123 or 10e+42
    fn digit(&mut self) -> Result<TokenType, LexError> {
        let mut s = String::new();

        // integer part
        self.push_while(&mut s, is_digit);

        // decmal part
        if let Some('.') = self.peek() {
            match self.decimal() {
                Ok(d) => s.push_str(&d),
                Err(e) => return Err(e),
            }
        }

        // exponent part
        if let Some('e') | Some('E') = self.peek() {
            match self.exponent() {
                Ok(e)  => s.push_str(&e),
                Err(e) => return Err(e),
            }
        }

        s.shrink_to_fit();
        Ok(TokenType::Number(s))
    }

    fn decimal(&mut self) -> Result<String, LexError> {
        self.skip();
        let mut s = String::new();
        match self.peek() {
            Some(c) if is_digit(c) => {
                self.skip();
                s.push(c);
            },
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        self.push_while(&mut s, is_digit);

        s.shrink_to_fit();
        Ok(s)
    }

    fn exponent(&mut self) -> Result<String, LexError> {
        self.skip();
        let mut s = String::new();
        match self.peek() {
            Some(c) if c == '+' || c == '-' => {
                self.skip();
                s.push(c);
            },
            _ => (),
        }

        match self.next_char() {
            Some(d) if is_digit(d) => s.push(d),
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        self.push_while(&mut s, is_digit);

        s.shrink_to_fit();
        Ok(s)
    }

    fn slash(&mut self, loc: Location) -> Result<Token, LexError> {
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
            // Line continuation
            Some(c) if is_line_terminator(c) => {
                if c == '\u{000D}' && self.peek() == Some('\u{000A}') {
                    self.skip();
                    let s = "\\\u{000D}\u{000A}".to_string();
                    Ok(s)
                } else {
                    Ok(format!("\\{}", c))
                }
            },
            // CharacterEscapeSequence
            Some(c) if is_escapable_char(c) => Ok(format!("\\{}", c)),
            // 0
            Some('0') => {
                Err(self.unexpected_eof())
            },
            // HexEscapeSequence
            Some(c) if c == '\\' => Ok(format!("\\\\")),
            // UnicodeEscapeSequence
            Some(c) => Ok(format!("\\{}", c)),
            None => Err(self.unexpected_eof()),
        }
    }

    fn equal(&mut self) -> TokenType {
        if self.skip_if('>') {
            TokenType::Arrow
        } else if self.skip_if('=') {
            if self.skip_if('=') {
                TokenType::TripleEquals
            } else {
                TokenType::DoubleEquals
            }
        } else {
            TokenType::Equals
        }
    }

    fn bang(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('=') {
            if self.skip_if('=') {
                TokenType::NotTripleEquals
            } else {
                TokenType::NotEquals
            }
        } else {
            TokenType::Bang
        }
    }

    fn ampersand(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('&') {
            TokenType::LogicalAnd
        } else if self.skip_if('=') {
            TokenType::BinaryAndEquals
        } else {
            TokenType::BinaryAnd
        }
    }

    fn pipe(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('|') {
            TokenType::LogicalOr
        } else if self.skip_if('=') {
            TokenType::BinaryOrEquals
        } else {
            TokenType::BinaryOr
        }
    }

    fn caret(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('=') {
            TokenType::BinaryXorEquals
        } else {
            TokenType::BinaryXor
        }
    }

    fn minus(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('-') {
            TokenType::Decrement
        } else if self.skip_if('=') {
            TokenType::MinusEquals
        } else {
            TokenType::Minus
        }
    }

    fn plus(&mut self) -> TokenType {
        self.skip();
        if self.skip_if('+') {
            TokenType::Increment
        } else if self.skip_if('=') {
            TokenType::PlusEquals
        } else {
            TokenType::Plus
        }
    }

    fn gt(&mut self) -> TokenType {
        self.skip();

        if self.skip_if('=') {
            TokenType::GreaterThanEqualTo
        } else if self.skip_if('>') {
            if self.skip_if('>') {
                if self.skip_if('=') {
                    TokenType::TripleRightShiftEquals
                } else {
                    TokenType::TripleRightShift
                }
            } else if self.skip_if('=') {
                TokenType::RightShiftEquals
            } else {
                TokenType::RightShift
            }
        } else {
            TokenType::GreaterThan
        }
    }

    fn lt(&mut self) -> TokenType {
        if self.skip_if('<') {
            if self.skip_if('=') {
                TokenType::LeftShiftEquals
            } else {
                TokenType::LeftShift
            }
        } else if self.skip_if('=') {
            TokenType::LessThanEqualTo
        } else {
            TokenType::LessThan
        }
    }

    fn period(&mut self, loc: Location) -> Result<Token, LexError> {
        self.skip();
        match self.peek() {
            Some('.') => {
                self.skip();
                match self.next_char() {
                    Some('.') => Ok(Token::new(loc, TokenType::Ellipsis)),
                    Some(c)   => Err(self.unexpected_char(c)),
                    None      => Err(self.unexpected_eof()),
                }
            },
            Some(c) if is_digit(c) => {
                self.decimal().map(|x| Token::new(loc, TokenType::Number(x)))
            },
            _ => Ok(Token::new(loc, TokenType::Period))
        }
    }

    fn percent(&mut self) -> TokenType {
        if self.skip_if('=') {
            TokenType::PercentEquals
        } else {
            TokenType::Percent
        }
    }

    fn asterisk(&mut self) -> TokenType {
        if self.skip_if('*') {
            if self.skip_if('=') {
                TokenType::PowerEquals
            } else {
                TokenType::Power
            }
        } else if self.skip_if('=') {
            TokenType::TimesEquals
        } else {
            TokenType::Times
        }
    }

    fn identifier(&mut self, loc: Location) -> Result<Token, LexError> {
        let mut s = String::new();

        // Head char
        match self.peek() {
            Some('\\') => {
                match self.unicode_escape() {
                    Ok(x)  => s.push_str(&x),
                    Err(e) => return Err(e),
                }
            },
            Some(c) if is_id_start(c) => {
                self.skip();
                s.push(c);
            },
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        // Tail chars
        loop {
            match self.peek() {
                Some('\\') => {
                    match self.unicode_escape() {
                        Ok(x)  => s.push_str(&x),
                        Err(e) => return Err(e),
                    }
                },
                Some(c) if is_id_continue(c) => {
                    self.skip();
                    s.push(c);
                },
                Some(_) |
                None => break
            }
        }

        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::Identifier(s)))
    }

    fn unicode_escape(&mut self) -> Result<String, LexError> {
        let mut s = String::with_capacity(6);

        // "\"
        s.push('\\');
        self.skip();

        // "u"
        match self.peek() {
            Some('u') => {
                s.push('u');
                self.skip()
            },
            Some(c)   => return Err(self.unexpected_char(c)),
            None      => return Err(self.unexpected_eof()),
        }

        match self.peek() {
            Some('{') => {
                // {CodePoint}
                s.push('{');

                // code point hex chars
                let mut cp = String::with_capacity(5);
                loop {
                    match self.peek() {
                        Some(c) if c.is_ascii_hexdigit() => {
                            cp.push(c);
                            s.push(c);
                        },
                        _ => break
                    }
                }
                // code point must be <= 0x10FFFF
                match cp.parse::<u32>() {
                    Ok(x) => {
                        if x > 0x10FFFF {
                            return Err(LexError::InvalidCodePoint(cp))
                        }
                    },
                    Err(_) => return Err(LexError::InvalidCodePoint(cp))
                }

                // Closing brace
                match self.peek() {
                    Some('}') => {
                        s.push('}');
                        self.skip();
                    },
                    Some(c) => return Err(self.unexpected_char(c)),
                    None    => return Err(self.unexpected_eof()),
                }
            },
            _ => {
                // Hex4Digits
                for _i in 0..4 {
                    match self.peek() {
                        Some(c) if c.is_ascii_hexdigit() => {
                            self.skip();
                            s.push(c);
                        },
                        Some(c) => return Err(self.unexpected_char(c)),
                        None    => return Err(self.unexpected_eof()),
                    }
                }
            }
        }

        s.shrink_to_fit();
        Ok(s)
    }
}

fn is_line_terminator(c: char) -> bool {
    match c {
        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

fn is_ws(c: char) -> bool {
    match c {
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' |
        '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}' |
        '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' |
        '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' |
        '\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{FEFF}' => true,
        _ => false,
    }
}

fn is_digit(c: char) -> bool {
    match c {
        '0' | '1' | '2' | '3' | '4' |
        '5' | '6' | '7' | '8' | '9' => true,
        _ => false,
    }
}

fn is_id_start(c: char) -> bool {
    match c {
        c if is(c, UnicodeProperty::IdStart) => true,
        '\\' => true,
        '$'  => true,
        '_'  => true,
        _    => false,
    }
}

fn is_id_continue(c: char) -> bool {
    match c {
        c if is(c, UnicodeProperty::IdContinue) => true,
        '\\'       => true,
        '$'        => true,
        '\u{200C}' => true, // ZWNJ
        '\u{200D}' => true, // ZWJ
        _          => false,
    }
}

fn is_escapable_char(c: char) -> bool {
    match c {
        '\'' | '"' | '\\' | 'b' | 'f' |
        'n'  | 'r' | 't'  | 'v' => true,
        _ => false,
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
                x if is_ws(x)       => Ok(Token::new(loc, self.ws())),
                x if is_digit(x)    => self.digit().map(|x| Token::new(loc, x)),
                x if is_id_start(x) => self.identifier(loc),
                '/'  => self.slash(loc),
                '\'' => self.string(QuoteStyle::Single).map(|x| Token::new(loc, x)),
                '"'  => self.string(QuoteStyle::Double).map(|x| Token::new(loc, x)),
                '{'  => Ok(self.scalar(loc, TokenType::LeftBrace)),
                '}'  => Ok(self.scalar(loc, TokenType::RightBrace)),
                '='  => Ok(Token::new(loc, self.equal())),
                '!'  => Ok(Token::new(loc, self.bang())),
                '&'  => Ok(Token::new(loc, self.ampersand())),
                '|'  => Ok(Token::new(loc, self.pipe())),
                '^'  => Ok(Token::new(loc, self.caret())),
                '('  => Ok(self.scalar(loc, TokenType::LeftParen)),
                ')'  => Ok(self.scalar(loc, TokenType::RightParen)),
                ']'  => Ok(self.scalar(loc, TokenType::RightBracket)),
                '['  => Ok(self.scalar(loc, TokenType::LeftBracket)),
                ':'  => Ok(self.scalar(loc, TokenType::Colon)),
                ','  => Ok(self.scalar(loc, TokenType::Comma)),
                '+'  => Ok(Token::new(loc, self.plus())),
                '-'  => Ok(Token::new(loc, self.minus())),
                '>'  => Ok(Token::new(loc, self.gt())),
                '<'  => Ok(Token::new(loc, self.lt())),
                '.'  => self.period(loc),
                '%'  => Ok(Token::new(loc, self.percent())),
                '*'  => Ok(Token::new(loc, self.asterisk())),
                '~'  => Ok(self.scalar(loc, TokenType::Tilde)),
                ';'  => Ok(self.scalar(loc, TokenType::Semicolon)),
                '?'  => Ok(self.scalar(loc, TokenType::Question)),
                c    => Err(self.unexpected_char(c)),
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
