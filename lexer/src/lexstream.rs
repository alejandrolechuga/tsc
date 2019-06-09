use std::iter::Peekable;

use super::location::Location;

pub struct LexStream<I>
where I: Iterator<Item = char>
{
    line: u32,
    column: u32,
    stream: Peekable<I>,
}

impl<I> LexStream<I>
where I: Iterator<Item = char>
{
    pub fn new(i: I) -> LexStream<I> {
        LexStream {
            line: 1,
            column: 1,
            stream: i.peekable()
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.stream.peek().map(|x| *x)
    }

    pub fn location(&self) -> Location {
        (self.line, self.column)
    }

    pub fn skip_char(&mut self) {
        self.next();
    }

    pub fn skip_if(&mut self, c: char) -> bool {
        match self.peek() {
            Some(x) if x == c => {
                self.skip_char();
                true
            },
            _ => false
        }
    }

    pub fn push_while<P>(&mut self, s: &mut String, predicate: P)
    where P: Fn(char) -> bool
    {
        loop {
            match self.peek() {
                Some(c) if predicate(c) => {
                    s.push(c);
                    self.skip_char();
                },
                _ => break
            }
        }
    }
}

impl<I> Iterator for LexStream<I>
where I: Iterator<Item = char>
{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.stream.next() {
            Some('\n') => {
                self.line += 1;
                self.column = 1;
                Some('\n')
            },
            Some(c) => {
                self.column += 1;
                Some(c)
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_first_char() {
        let stream = LexStream::new("abc");
        let next = stream.next();
        assert_eq!('a', next);
    }
}
