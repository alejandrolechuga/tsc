extern crate regex;

mod lexer;

use lexer::Lexer;

#[cfg(not(test))]
fn main() {
    let l = Lexer::new("let i = 0;".chars());
    for t in l {
        println!("{:?}", t.unwrap());
    }
}
