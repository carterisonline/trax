#[rustfmt::skip]
mod test {
    use crate::token::*;

    test!(document_01, "",);

    test!(document_02, "    ",);

    test!(document_03, " \n\t\r ",);

    // BOM
    test!(document_05, core::str::from_utf8(b"\xEF\xBB\xBF<a/>").unwrap(),
        Token::ElementStart("", "a", 3..5),
        Token::ElementEnd(ElementEnd::Empty, 5..7)
    );

    test!(document_err_01, "<![CDATA[text]]>",
        Token::Error("invalid element at 1:1 cause invalid name token".to_string())
    );

    test!(document_err_02, " &www---------Ӥ+----------w-----www_",
        Token::Error("unknown token at 1:2".to_string())
    );

    test!(document_err_03, "q",
        Token::Error("unknown token at 1:1".to_string())
    );

    test!(document_err_04, "<!>",
        Token::Error("invalid element at 1:1 cause invalid name token".to_string())
    );

    test!(document_err_06, "&#x20;",
        Token::Error("unknown token at 1:1".to_string())
    );

    #[test]
    fn parse_fragment_1() {
        let s = "<p/><p/>";
        let mut p = trax_parser::Tokenizer::from_fragment(s, 0..s.len());

        match p.next().unwrap().unwrap() {
            trax_parser::Token::ElementStart { local, .. } => assert_eq!(local.as_str(), "p"),
            _ => panic!(),
        }

        match p.next().unwrap().unwrap() {
            trax_parser::Token::ElementEnd { .. } => {}
            _ => panic!(),
        }

        match p.next().unwrap().unwrap() {
            trax_parser::Token::ElementStart { local, .. } => assert_eq!(local.as_str(), "p"),
            _ => panic!(),
        }
    }
}
