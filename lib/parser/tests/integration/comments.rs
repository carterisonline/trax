use crate::token::*;

test!(comment_01, "/*comment*/", Token::Comment("comment", 0..11));
test!(comment_02, "/*<head>*/", Token::Comment("<head>", 0..10));
test!(comment_03, "/*/*x*/", Token::Comment("/*x", 0..7));
test!(comment_04, "/*/x*/", Token::Comment("/x", 0..6));
test!(comment_05, "/**/", Token::Comment("", 0..4));
test!(comment_06, "/*\n*\n*\n*/", Token::Comment("\n*\n*\n", 0..9));

macro_rules! test_err {
    ($name:ident, $text:expr) => {
        #[test]
        fn $name() {
            let mut p = trax_parser::Tokenizer::from($text);
            assert!(p.next().unwrap().is_err());
        }
    };
}

test_err!(comment_err_01, "/*");
test_err!(comment_err_02, "*/");
test_err!(comment_err_03, "/*/");
test_err!(comment_err_04, "*/*");
test_err!(comment_err_05, "//*");
test_err!(comment_err_06, "//**/");
test_err!(comment_err_07, "*/");
test_err!(comment_err_08, "**/");
