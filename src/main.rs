#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub ecma);

#[test]
fn input_element_div() {
    let comment = ecma::InputElementDivParser::new().parse("//this is a comment").unwrap();
    assert_eq!(comment, "//this is a comment");
}

#[test]
fn comment_newline() {
    let comment = ecma::InputElementDivParser::new().parse("//comment\n").unwrap();
    assert_eq!(comment, "//comment");
}

// #[test]
// fn ws() {
//     let ws = ecma::InputElementDivParser::new().parse(" ").unwrap();
//     assert_eq!(ws, " ");
// }

//#[test]
//fn line_terminator() {
//    let lt = ecma::InputElementDivParser::new().parse("\n").unwrap();
//    assert_eq!(lt, "\n");
//}

#[cfg(not(test))]
fn main() {
    println!("Hello world");
}
