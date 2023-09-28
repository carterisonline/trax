type Range = core::ops::Range<usize>;

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    Comment(&'a str, Range),
    ElementStart(&'a str, &'a str, Range),
    Attribute(&'a str, &'a str, &'a str, Range),
    Modifier(&'a str, &'a str, Range),
    ElementEnd(ElementEnd<'a>, Range),
    Text(&'a str, Range),
    Error(String),
}

#[derive(PartialEq, Debug)]
pub enum ElementEnd<'a> {
    Open,
    Close(&'a str, &'a str),
    Empty,
}

#[macro_export]
macro_rules! test {
    ($name:ident, $text:expr, $($token:expr),*) => (
        #[test]
        fn $name() {
            let mut p = trax_parser::Tokenizer::from($text);
            $(
                let t = p.next().unwrap();
                assert_eq!(to_test_token(t), $token);
            )*
            assert!(p.next().is_none());
        }
    )
}

#[inline(never)]
pub fn to_test_token(token: Result<trax_parser::Token, trax_parser::Error>) -> Token {
    match token {
        Ok(trax_parser::Token::Comment { text, span }) => {
            Token::Comment(text.as_str(), span.range())
        }
        Ok(trax_parser::Token::ElementStart {
            prefix,
            local,
            span,
        }) => Token::ElementStart(prefix.as_str(), local.as_str(), span.range()),
        Ok(trax_parser::Token::Attribute {
            prefix,
            local,
            value,
            span,
        }) => Token::Attribute(
            prefix.as_str(),
            local.as_str(),
            value.as_str(),
            span.range(),
        ),
        Ok(trax_parser::Token::Modifier {
            prefix,
            local,
            span,
        }) => Token::Modifier(prefix.as_str(), local.as_str(), span.range()),
        Ok(trax_parser::Token::ElementEnd { end, span }) => Token::ElementEnd(
            match end {
                trax_parser::ElementEnd::Open => ElementEnd::Open,
                trax_parser::ElementEnd::Close(prefix, local) => {
                    ElementEnd::Close(prefix.as_str(), local.as_str())
                }
                trax_parser::ElementEnd::Empty => ElementEnd::Empty,
            },
            span.range(),
        ),
        Ok(trax_parser::Token::Text { text }) => Token::Text(text.as_str(), text.range()),
        Err(ref e) => Token::Error(e.to_string()),
    }
}
