#![feature(macro_metavar_expr)]

mod drop;
mod parse;

// This macro **should** work but I'm using an experimental method to
// generate macros that generate macros that generate macros tha-
// ... so maybe that should've been expected.
// Works when using "inline macro" with Rust Analyzer though...

#[macro_export]
macro_rules! test_suite {
    ($group: ident, $($r: tt)+) => {
        macro_rules! impl_test_suite {
            ($$($x: tt)+) => {
                macro_rules! $group {
                    ($name: ident, $i: literal, $($r)+) => {
                        paste::paste! {
                            #[test]
                            fn [<$group _ $name _ $i>]() {
                                let mut doc = trax_document::Document::new(include_str!(concat!("../testfiles/", stringify!($name), ".trax"))).unwrap();

                                $$($$x)+

                                assert_eq!(
                                    &doc.into_string(),
                                    include_str!(concat!("../testfiles/", stringify!($group), "/", stringify!($name), ".", stringify!($i), ".trax")),
                                );
                            }
                        }
                    };
                }
            }
        }
    }
}
