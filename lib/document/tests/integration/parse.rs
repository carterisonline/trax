macro_rules! parse_document_err {
    ($name: ident, $text: expr, $err: ident($($i: tt)*)) => {
        #[test]
        fn $name() {
            assert_eq!(Document::new($text), Err(DocumentParseError::$err($($i)*)));
        }
    };

    ($name: ident, $text: expr, $err: ident{$($i: tt)*}) => {
        #[test]
        fn $name() {
            assert_eq!(Document::new($text), Err(DocumentParseError::$err{$($i)*}));
        }
    };

    ($name: ident, $text: expr, $err: ident) => {
        #[test]
        fn $name() {
            assert_eq!(Document::new($text), Err(DocumentParseError::$err));
        }
    };
}

macro_rules! range {
    ($r1: expr, $c1: literal..$c2: literal) => {
        TextRange::new(TextPos::new($r1, $c1), TextPos::new($r1, $c2))
    };

    ($r1: expr, $c1: expr, $r2: expr, $c2: expr) => {
        TextRange::new(TextPos::new($r1, $c1), TextPos::new($r2, $c2))
    };
}

#[cfg(test)]
mod test {
    use trax_document::{Document, DocumentParseError};
    use trax_parser::{TextPos, TextRange};

    parse_document_err!(err_empty_document, "", EmptyDocument);

    parse_document_err!(
        err_invalid_root_element_1,
        "<doxument> </doxument>",
        InvalidRootElement(range!(1, 1..10))
    );

    parse_document_err!(
        err_invalid_root_element_2,
        "<prefix:document> </prefix:document>",
        InvalidRootElement(range!(1, 1..17))
    );

    parse_document_err!(
        err_invalid_root_element_3,
        "/* comment */",
        InvalidRootElement(range!(1, 1..14))
    );

    parse_document_err!(
        err_invalid_tree_structure,
        "<document><one><two></one></two></document>",
        InvalidTreeStructure {
            closed_elem: "one".into(),
            current_open_elem: "two".into(),
            location: range!(1, 21..27)
        }
    );

    #[test]
    fn can_reproduce_input() {
        let src = include_str!("../../../../doc/todo.trax");
        assert_eq!(Document::new(src).unwrap().into_string(), src);
    }
}
