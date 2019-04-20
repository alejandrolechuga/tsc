#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub ecma);

#[test]
fn input_element_div() {
    ecma::InputElementDivParser::new().parse("  // this is a comment").unwrap()
}

#[cfg(not(test))]
fn main() {
    println!("Hello world");
}
