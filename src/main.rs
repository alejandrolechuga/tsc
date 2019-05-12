#[macro_use] extern crate lazy_static;
extern crate regex;

mod lexer;

#[cfg(not(test))]
fn main() {
    println!("Hello world");
}
