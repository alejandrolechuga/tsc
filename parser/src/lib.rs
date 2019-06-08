extern crate lexer;

use lexer::Lexer;

pub enum Node {
    Boolean,
}

pub struct Parser<I>
where I: Iterator<Item = char> {
    _lexer: Lexer<I>,
}

impl<I> Parser<I>
where I: Iterator<Item = char> {
    pub fn new(lexer: Lexer<I>) -> Parser<I> {
        Parser { _lexer: lexer }
    }

    pub fn parse() -> Node {
        Node::Boolean
    }
}
