// pub mod ast;
//
// #[macro_use]
// extern crate lalrpop_util;
//
// lalrpop_mod!(pub ecma);
//
// #[test]
// fn comment_single() {
//     assert!(ecma::InputElementDivParser::new().parse("//this is a comment").is_ok());
//     assert!(ecma::InputElementDivParser::new().parse("//comment\n").is_ok());
// }
//
// #[test]
// fn comment_multi() {
//     assert!(ecma::InputElementDivParser::new().parse("/**/").is_ok());
//     assert!(ecma::InputElementDivParser::new().parse("/*comment*/").is_ok());
//     assert!(ecma::InputElementDivParser::new().parse("/*com*m/ent*/").is_ok());
//     assert!(ecma::InputElementDivParser::new().parse("/*com\nment*/").is_ok());
// }

mod lexer;

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
