extern crate unicode;

mod charclass;
mod lexerror;
mod lexstream;
mod location;
mod token;

use self::location::Location;
use self::lexstream::LexStream;
use self::token::{CommentStyle,QuoteStyle,Token,TokenType};
use self::lexerror::LexError;

use self::charclass::{
    is_escapable_char,
    is_id_start,
    is_id_continue,
    is_line_terminator,
    is_ws,
};

pub enum LexGoal {
    InputElementDiv,
    InputElementRegExp,
}

pub struct Lexer<I>
where I: Iterator<Item = char>,
{
    goal:   LexGoal,
    stream: LexStream<I>,
}

impl<I> Lexer<I>
where I: Iterator<Item = char>,
{
    pub fn new(stream: I) -> Lexer<I> {
        Lexer {
            goal:   LexGoal::InputElementDiv,
            stream: LexStream::new(stream),
        }
    }

    fn unexpected_char(&self, c: char) -> LexError {
        let (line, col) = self.stream.location();
        LexError::UnexpectedCharacter(line, col, c)
    }

    fn unexpected_eof(&self) -> LexError {
        let (line, col) = self.stream.location();
        LexError::UnexpectedEndOfInput(line, col)
    }

    fn scalar(&mut self, loc: Location, typ: TokenType) -> Token {
        self.stream.skip_char();
        Token::new(loc, typ)
    }

    pub fn set_goal(&mut self, goal: LexGoal) {
        self.goal = goal;
    }

    fn comment(&mut self, loc: Location, style: CommentStyle) -> Result<Token, LexError> {
        let mut s = String::new();
        self.stream.skip_char();
        match style {
            CommentStyle::SingleLine => self.stream.push_while(&mut s, |x| !is_line_terminator(x)),
            CommentStyle::MultiLine => {
                loop {
                    match self.stream.next() {
                        Some('*') => {
                            match self.stream.next() {
                                Some('/') => break,
                                Some(c) => {
                                    s.push('*');
                                    s.push(c);
                                },
                                None => return Err(self.unexpected_eof()),
                            }
                        },
                        Some(c) => s.push(c),
                        None    => return Err(self.unexpected_eof()),
                    }
                }
            },
        }
        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::Comment(s, style)))
    }

    fn ws(&mut self) -> TokenType {
        let mut s = String::new();
        self.stream.push_while(&mut s, is_ws);
        s.shrink_to_fit();
        TokenType::WhiteSpace(s)
    }

    fn string(&mut self, quote: QuoteStyle) -> Result<TokenType, LexError> {
        let mut s = String::new();
        self.stream.skip_char();

        loop {
            match self.stream.next() {
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
                None    => return Err(self.unexpected_eof()),
            }
        }

        s.shrink_to_fit();
        Ok(TokenType::String(s, quote))
    }

    // matches constructs beginning with a digit, e.g. 0.123 or 10e+42
    fn digit(&mut self) -> Result<TokenType, LexError> {
        let mut s = String::new();

        // is digit 0
        // is the next thing x
        // next thing should hex digits
        match self.stream.peek() {
            Some('0') => {
                s.push('0');
                self.stream.skip_char();

                match self.stream.peek() {
                    Some(c) if c == 'x' || c == 'X' => {
                        self.stream.skip_char();
                        s.push(c);
                        match self.stream.next() {
                            Some(c) if c.is_ascii_hexdigit() => {
                                s.push(c);
                            },
                            Some(c) => {
                                return Err(self.unexpected_char(c))
                            },
                            None => return Err(self.unexpected_eof())
                        }
                        self.stream.push_while(&mut s, |c| c.is_ascii_hexdigit());
                        return Ok(TokenType::Number(s));
                    },
                    Some(_) => (),
                    None => ()
                }
            },
            None => return Err(self.unexpected_eof()),
            Some(_) => ()
        }

        // integer part
        self.stream.push_while(&mut s, |c| c.is_ascii_digit());

        // decimal part
        if let Some('.') = self.stream.peek() {
            self.stream.skip_char();
            match self.decimal() {
                Ok(d) => s.push_str(&d),
                Err(e) => return Err(e),
            }
        }

        // exponent part
        if let Some('e') | Some('E') = self.stream.peek() {
            match self.exponent() {
                Ok(e)  => s.push_str(&e),
                Err(e) => return Err(e),
            }
        }

        s.shrink_to_fit();
        Ok(TokenType::Number(s))
    }

    // Assumes the "." has been skipped
    fn decimal(&mut self) -> Result<String, LexError> {
        // self.stream.skip_char();
        let mut s = String::new();
        s.push('.');
        match self.stream.next() {
            Some(c) if c.is_ascii_digit() => s.push(c),
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        self.stream.push_while(&mut s, |c| c.is_ascii_digit());

        s.shrink_to_fit();
        Ok(s)
    }

    fn exponent(&mut self) -> Result<String, LexError> {
        self.stream.skip_char();
        let mut s = String::new();
        match self.stream.peek() {
            Some(c) if c == '+' || c == '-' => {
                self.stream.skip_char();
                s.push(c);
            },
            _ => (),
        }

        match self.stream.next() {
            Some(d) if d.is_ascii_digit() => s.push(d),
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        self.stream.push_while(&mut s, |d| d.is_ascii_digit());

        s.shrink_to_fit();
        Ok(s)
    }

    fn slash(&mut self, loc: Location) -> Result<Token, LexError> {
        self.stream.skip_char();
        match self.stream.peek() {
            Some('/') => self.comment(loc, CommentStyle::SingleLine),
            Some('*') => self.comment(loc, CommentStyle::MultiLine),
            Some('=') => Ok(self.scalar(loc, TokenType::DivEqual)),
            None |
            Some(_)   => Ok(self.scalar(loc, TokenType::Div)),
        }
    }

    fn escape_or_line_continuation(&mut self) -> Result<String, LexError> {
        match self.stream.next() {
            // Line continuation
            Some(c) if is_line_terminator(c) => {
                if c == '\u{000D}' && self.stream.peek() == Some('\u{000A}') {
                    self.stream.skip_char();
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
                self.stream.skip_char();
                match self.stream.peek() {
                    Some(c) if c.is_ascii_digit() => Err(self.unexpected_char(c)),
                    _ => Ok("\\0".to_string()),
                }
            },
            // HexEscapeSequence
            Some('x') => {
                let mut s = String::with_capacity(4);
                s.push('\\');
                s.push('x');
                self.stream.skip_char();

                match self.stream.peek() {
                    Some(c) if c.is_ascii_hexdigit() => {
                        s.push(c);
                        self.stream.skip_char();
                        match self.stream.peek() {
                            Some(c) if c.is_ascii_hexdigit() => {
                                self.stream.skip_char();
                                s.push(c);
                                Ok(s)
                            },
                            Some(c) => Err(self.unexpected_char(c)),
                            None    => Err(self.unexpected_eof()),
                        }
                    },
                    Some(x) => Err(self.unexpected_char(x)),
                    None    => Err(self.unexpected_eof()),
                }
            },
            // UnicodeEscapeSequence
            Some('u') => self.unicode_escape(),
            // Something else; invalid
            Some(c) => Err(self.unexpected_char(c)),
            None    => Err(self.unexpected_eof()),
        }
    }

    fn template(&mut self, loc: Location) -> Result<Token, LexError> {
        self.stream.skip_char();
        let mut s = String::new();

        loop {
            match self.stream.next() {
                Some('$') => {
                    s.push('$');
                    match self.stream.peek() {
                        Some('{') => {
                            s.push('{');
                            self.stream.skip_char();
                            break;
                        },
                        Some(c) => {
                            self.stream.skip_char();
                            s.push(c);
                        },
                        None => return Err(self.unexpected_eof()),
                    }
                },
                Some('\\') => {
                    s.push('\\');
                    match self.stream.next() {
                        Some(c) => s.push(c),
                        None    => return Err(self.unexpected_eof()),
                    }
                },
                Some('`') => {
                    self.stream.skip_char();
                    s.push('`');
                    break;
                },
                Some(c) => {
                    s.push(c);
                }
                None => return Err(self.unexpected_eof()),
            }
        }

        s.shrink_to_fit();
        Ok(Token::new(loc, TokenType::Template(s)))
    }

    fn equal(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('>') {
            TokenType::Arrow
        } else if self.stream.skip_if('=') {
            if self.stream.skip_if('=') {
                TokenType::TripleEquals
            } else {
                TokenType::DoubleEquals
            }
        } else {
            TokenType::Equals
        }
    }

    fn bang(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('=') {
            if self.stream.skip_if('=') {
                TokenType::NotTripleEquals
            } else {
                TokenType::NotEquals
            }
        } else {
            TokenType::Bang
        }
    }

    fn ampersand(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('&') {
            TokenType::LogicalAnd
        } else if self.stream.skip_if('=') {
            TokenType::BinaryAndEquals
        } else {
            TokenType::BinaryAnd
        }
    }

    fn pipe(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('|') {
            TokenType::LogicalOr
        } else if self.stream.skip_if('=') {
            TokenType::BinaryOrEquals
        } else {
            TokenType::BinaryOr
        }
    }

    fn caret(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('=') {
            TokenType::BinaryXorEquals
        } else {
            TokenType::BinaryXor
        }
    }

    fn minus(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('-') {
            TokenType::Decrement
        } else if self.stream.skip_if('=') {
            TokenType::MinusEquals
        } else {
            TokenType::Minus
        }
    }

    fn plus(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('+') {
            TokenType::Increment
        } else if self.stream.skip_if('=') {
            TokenType::PlusEquals
        } else {
            TokenType::Plus
        }
    }

    fn gt(&mut self) -> TokenType {
        self.stream.skip_char();

        if self.stream.skip_if('=') {
            TokenType::GreaterThanEqualTo
        } else if self.stream.skip_if('>') {
            if self.stream.skip_if('>') {
                if self.stream.skip_if('=') {
                    TokenType::TripleRightShiftEquals
                } else {
                    TokenType::TripleRightShift
                }
            } else if self.stream.skip_if('=') {
                TokenType::RightShiftEquals
            } else {
                TokenType::RightShift
            }
        } else {
            TokenType::GreaterThan
        }
    }

    fn lt(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('<') {
            if self.stream.skip_if('=') {
                TokenType::LeftShiftEquals
            } else {
                TokenType::LeftShift
            }
        } else if self.stream.skip_if('=') {
            TokenType::LessThanEqualTo
        } else {
            TokenType::LessThan
        }
    }

    fn period(&mut self, loc: Location) -> Result<Token, LexError> {
        self.stream.skip_char();
        match self.stream.peek() {
            Some('.') => {
                self.stream.skip_char();
                match self.stream.next() {
                    Some('.') => Ok(Token::new(loc, TokenType::Ellipsis)),
                    Some(c)   => Err(self.unexpected_char(c)),
                    None      => Err(self.unexpected_eof()),
                }
            },
            Some(c) if c.is_ascii_digit() => {
                self.decimal().map(|x| Token::new(loc, TokenType::Number(x)))
            },
            _ => Ok(Token::new(loc, TokenType::Period))
        }
    }

    fn percent(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('=') {
            TokenType::PercentEquals
        } else {
            TokenType::Percent
        }
    }

    fn asterisk(&mut self) -> TokenType {
        self.stream.skip_char();
        if self.stream.skip_if('*') {
            if self.stream.skip_if('=') {
                TokenType::PowerEquals
            } else {
                TokenType::Power
            }
        } else if self.stream.skip_if('=') {
            TokenType::TimesEquals
        } else {
            TokenType::Times
        }
    }

    fn identifier(&mut self, loc: Location) -> Result<Token, LexError> {
        let mut s = String::new();

        // Head char
        match self.stream.peek() {
            Some('\\') => {
                match self.unicode_escape() {
                    Ok(x)  => s.push_str(&x),
                    Err(e) => return Err(e),
                }
            },
            Some(c) if is_id_start(c) => {
                self.stream.skip_char();
                s.push(c);
            },
            Some(c) => return Err(self.unexpected_char(c)),
            None    => return Err(self.unexpected_eof()),
        }

        // Tail chars
        loop {
            match self.stream.peek() {
                Some('\\') => {
                    match self.unicode_escape() {
                        Ok(x)  => s.push_str(&x),
                        Err(e) => return Err(e),
                    }
                },
                Some(c) if is_id_continue(c) => {
                    self.stream.skip_char();
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
        self.stream.skip_char();

        // "u"
        match self.stream.peek() {
            Some('u') => {
                s.push('u');
                self.stream.skip_char();
            },
            Some(c)   => return Err(self.unexpected_char(c)),
            None      => return Err(self.unexpected_eof()),
        }

        match self.stream.peek() {
            Some('{') => {
                // {CodePoint}
                s.push('{');

                // code point hex chars
                let mut cp = String::with_capacity(5);
                loop {
                    match self.stream.peek() {
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
                match self.stream.peek() {
                    Some('}') => {
                        s.push('}');
                        self.stream.skip_char();
                    },
                    Some(c) => return Err(self.unexpected_char(c)),
                    None    => return Err(self.unexpected_eof()),
                }
            },
            _ => {
                // Hex4Digits
                for _i in 0..4 {
                    match self.stream.peek() {
                        Some(c) if c.is_ascii_hexdigit() => {
                            self.stream.skip_char();
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

    fn regex(&mut self, loc: Location) -> Result<Token, LexError> {
        self.stream.skip_char();
        let mut body = String::new();

        match self.stream.peek() {
            Some(c) if c == '*' || c == '/' => return Err(self.unexpected_char(c)),
            _ => (),
        }

        // body
        loop {
            match self.stream.next() {
                Some('/') => {
                    break
                },
                Some('[') => {
                    body.push('[');
                    loop {
                        match self.stream.next() {
                            Some(']')  => {
                                body.push(']');
                                break
                            },
                            Some('\\') => {
                                match self.stream.next() {
                                    Some(c) if is_line_terminator(c) => return Err(self.unexpected_char(c)),
                                    Some(c) => body.push(c),
                                    None    => return Err(self.unexpected_eof()),
                                }
                            },
                            Some(c) if is_line_terminator(c) => return Err(self.unexpected_char(c)),
                            Some(c) => body.push(c),
                            None => return Err(self.unexpected_eof()),
                        }
                    }
                },
                Some('\\') => {
                    body.push('\\');
                    match self.stream.next() {
                        Some(c) if is_line_terminator(c) => return Err(self.unexpected_char(c)),
                        Some(c) => body.push(c),
                        None    => return Err(self.unexpected_eof()),
                    }
                },
                Some(c) if is_line_terminator(c) => return Err(self.unexpected_char(c)),
                Some(c) => body.push(c),
                None    => return Err(self.unexpected_eof()),
            }
        }

        // flags
        let mut flags = String::new();
        loop {
            match self.stream.peek() {
                Some('\\') => {
                    match self.unicode_escape() {
                        Ok(esc) => flags.push_str(&esc),
                        Err(e)  => return Err(e),
                    }
                },
                Some(c) if is_id_continue(c) => {
                    flags.push(c);
                    self.stream.skip_char();
                },
                _ => break,
            }
        }

        body.shrink_to_fit();
        flags.shrink_to_fit();
        Ok(Token::new(loc, TokenType::RegExp(body, flags)))
    }
}

impl<I> Iterator for Lexer<I>
where I: Iterator<Item = char>
{
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Result<Token, LexError>> {
        let loc = self.stream.location();
        self.stream.peek().map(|next| {
            match next {
                x if is_ws(x)           => Ok(Token::new(loc, self.ws())),
                x if x.is_ascii_digit() => self.digit().map(|x| Token::new(loc, x)),
                x if is_id_start(x)     => self.identifier(loc),
                '`'  => self.template(loc),
                '/'  => {
                    match self.goal {
                        LexGoal::InputElementDiv => self.slash(loc),
                        LexGoal::InputElementRegExp => self.regex(loc),
                    }
                },
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
        verify_single(" \t\t ");
    }

    #[test]
    fn identifies_comments() {
        verify_single("//this is a comment");
    }

    #[test]
    fn identifies_single_strings() {
        verify_single("'this is a string'");
    }

    #[test]
    fn identifies_double_strings() {
        verify_single(r#""this is a string""#);
    }

    #[test]
    fn identifies_div() {
        verify_single("/");
    }

    #[test]
    fn identifies_div_equals() {
        verify_single("/=");
    }

    #[test]
    fn identifies_right_brace() {
        verify_single("}");
    }

    #[test]
    fn identifies_zero() {
        verify_single("0");
    }

    #[test]
    fn identifies_int() {
        verify_single("42");
    }

    #[test]
    fn identifies_decimal() {
        verify_single(".123");
    }

    #[test]
    fn identifies_zero_decimal() {
        verify_single("0.123");
    }

    #[test]
    fn identifies_regex() {
        let input = "/[0-9]/i";
        let mut lex = Lexer::new(input.chars());
        lex.set_goal(LexGoal::InputElementRegExp);
        let next = lex.next();
        assert!(lex.next().is_none());
        assert_eq!(token_text(next), input);
    }

    #[test]
    fn identifies_dot() {
        verify_single(".");
    }

    #[test]
    fn identifies_ellipsis() {
        verify_single("...");
    }

    #[test]
    fn identifies_assignment() {
        verify_single("=");
    }

    #[test]
    fn identifies_double_equals() {
        verify_single("==");
    }

    #[test]
    fn identifies_triple_equals() {
        verify_single("===");
    }

    #[test]
    fn identifies_arrow() {
        verify_single("=>");
    }

    #[test]
    fn identifies_lt() {
        verify_single("<");
    }

    #[test]
    fn identifies_lte() {
        verify_single("<=");
    }

    #[test]
    fn identifies_left_shift() {
        verify_single("<<");
    }

    #[test]
    fn identifies_left_shift_equals() {
        verify_single("<<=");
    }

    #[test]
    fn identifies_gt() {
        verify_single(">");
    }

    #[test]
    fn identifies_gte() {
        verify_single(">=");
    }

    #[test]
    fn identifies_right_shift() {
        verify_single(">>");
    }

    #[test]
    fn identifies_right_shift_equals() {
        verify_single(">>=");
    }

    #[test]
    fn identifies_triple_right_shift() {
        verify_single(">>>");
    }

    #[test]
    fn identifies_triple_right_shift_equals() {
        verify_single(">>>=");
    }

    #[test]
    fn identifies_identifiers() {
        verify_single("foo");
    }

    #[test]
    fn identifies_left_paren() {
        verify_single("(");
    }

    #[test]
    fn identifies_right_paren() {
        verify_single(")");
    }

    #[test]
    fn identifies_left_bracket() {
        verify_single("]");
    }

    #[test]
    fn identifies_right_bracket() {
        verify_single("]");
    }

    #[test]
    fn identifies_semicolon() {
        verify_single(";");
    }

    #[test]
    fn identifies_colon() {
        verify_single(":");
    }

    #[test]
    fn identifies_comma() {
        verify_single(",");
    }

    #[test]
    fn identifies_binary_and() {
        verify_single("&");
    }

    #[test]
    fn identifies_binary_and_equals() {
        verify_single("&=");
    }

    #[test]
    fn identifies_logical_and() {
        verify_single("&&");
    }

    #[test]
    fn identifies_binary_or() {
        verify_single("|");
    }

    #[test]
    fn identifies_binary_or_equals() {
        verify_single("|=");
    }

    #[test]
    fn identifies_logical_or() {
        verify_single("||");
    }

    #[test]
    fn identifies_bang() {
        verify_single("!");
    }

    #[test]
    fn identifies_not_equals() {
        verify_single("!=");
    }

    #[test]
    fn identifies_not_equals_equals() {
        verify_single("!==");
    }

    #[test]
    fn identifies_binary_xor() {
        verify_single("^");
    }

    #[test]
    fn identifies_binary_xor_equals() {
        verify_single("^=");
    }

    #[test]
    fn identifies_plus() {
        verify_single("+");
    }

    #[test]
    fn identifies_plus_equals() {
        verify_single("+=");
    }

    #[test]
    fn identifies_mod() {
        verify_single("%");
    }

    #[test]
    fn identifies_mod_equals() {
        verify_single("%=");
    }

    #[test]
    fn identifies_times() {
        verify_single("*");
    }

    #[test]
    fn identifies_times_equals() {
        verify_single("*=");
    }

    #[test]
    fn identifies_power() {
        verify_single("**");
    }

    #[test]
    fn identifies_power_equals() {
        verify_single("**=");
    }

    fn verify_single(input: &str) {
        let output = single_token(input);
        assert_eq!(token_text(output), input);
    }

    fn single_token(input: &str) -> Option<Result<Token, LexError>> {
        let mut lexer = Lexer::new(input.chars());
        let ret = lexer.next();
        assert!(lexer.next().is_none(), "lexed more than one token");
        ret
    }

    fn token_text(tok: Option<Result<Token, LexError>>) -> String {
        match tok {
            Some(Ok(t)) => t.typ.to_string(),
            Some(Err(e)) => panic!("{:?}", e),
            None => panic!("Didn't get a token from the lexer")
        }
    }
}
