use std::collections::VecDeque;

use trax_parser::{span_text_range as r, ElementEnd, TextRange, Token, Tokenizer};

#[derive(Debug, PartialEq)]
pub enum EntityType {
    Element,
    Text,
}

#[derive(Debug, Default, PartialEq)]
pub struct Attribute {
    prefix: String,
    tag: String,
    value: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Element {
    parent: usize,
    children: VecDeque<(EntityType, usize)>,
    prefix: String,
    tag: String,
    attributes: VecDeque<Attribute>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Document {
    element_store: Vec<Element>,
    text_store: Vec<String>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DocumentParseError {
    #[error("Can't parse an empty document")]
    EmptyDocument,

    #[error("at {0}: Document must have a single root <document> element")]
    InvalidRootElement(TextRange),

    #[error("syntax error")]
    SyntaxError(#[from] trax_parser::Error),

    #[error("invalid tree structure: `{closed_elem}` was closed ({location}) here but a different element `{current_open_elem}` was opened after it, and should be closed first")]
    InvalidTreeStructure {
        closed_elem: String,
        current_open_elem: String,
        location: TextRange,
    },
}

impl Document {
    pub fn new(source: &str) -> Result<Self, DocumentParseError> {
        let mut tokenizer = Tokenizer::from(source);

        validate_document_start(source, tokenizer.next())?;

        dbg!(tokenizer.next());

        let mut element_num = 1;
        let mut text_num = 0;
        let mut hierarchy = Vec::<usize>::new();
        let mut element_store = Vec::<Element>::new();
        let mut text_store = Vec::<String>::new();

        hierarchy.push(0);
        element_store.push(Element {
            tag: "document".into(),
            ..Default::default()
        });

        for token in tokenizer {
            let token = token?;

            match token {
                Token::ElementStart { prefix, local, .. } => {
                    let parent = *hierarchy.last().unwrap();

                    element_store[parent]
                        .children
                        .push_back((EntityType::Element, element_num));

                    hierarchy.push(element_num);
                    element_store.push(Element {
                        parent,
                        prefix: prefix.as_str().into(),
                        tag: local.as_str().into(),
                        ..Default::default()
                    });

                    element_num += 1;
                }

                Token::Attribute {
                    prefix,
                    local,
                    value,
                    ..
                } => element_store[*hierarchy.last().unwrap()]
                    .attributes
                    .push_back(Attribute {
                        prefix: prefix.as_str().into(),
                        tag: local.as_str().into(),
                        value: Some(value.as_str().into()),
                    }),

                Token::Modifier { prefix, local, .. } => element_store[*hierarchy.last().unwrap()]
                    .attributes
                    .push_back(Attribute {
                        prefix: prefix.as_str().into(),
                        tag: local.as_str().into(),
                        value: None,
                    }),

                Token::ElementEnd {
                    end: ElementEnd::Empty,
                    ..
                } => {
                    dbg!(&hierarchy);
                    hierarchy.pop();
                }

                // Ending the *current* open element
                Token::ElementEnd {
                    end: ElementEnd::Close(prefix, local),
                    ..
                } if element_store[*hierarchy.last().unwrap()].prefix == prefix.as_str()
                    && element_store[*hierarchy.last().unwrap()].tag == local.as_str() =>
                {
                    hierarchy.pop();
                }

                // Element is already in the hierarchy so we can ignore these
                Token::ElementEnd {
                    end: ElementEnd::Open,
                    ..
                } => (),

                // Closing an open element different from the current one makes an invalid tree
                // structure. Needed since the alloc-free parser doesn't do this.
                Token::ElementEnd {
                    end: ElementEnd::Close(prefix, local),
                    span,
                } => {
                    let current_open_elem = &element_store[*hierarchy.last().unwrap()];
                    dbg!(&element_store);
                    dbg!(&hierarchy);
                    return Err(DocumentParseError::InvalidTreeStructure {
                        closed_elem: gen_full_name(&prefix, &local),
                        current_open_elem: gen_full_name(
                            &current_open_elem.prefix,
                            &current_open_elem.tag,
                        ),
                        location: r(source, span),
                    });
                }

                Token::Text { text } => {
                    element_store[*hierarchy.last().unwrap()]
                        .children
                        .push_back((EntityType::Text, text_num));

                    text_store.push(text.as_str().into());

                    text_num += 1;
                }

                Token::Comment { .. } => (),
            }
        }

        Ok(Self {
            element_store,
            text_store,
        })
    }

    pub fn into_string(&self) -> String {
        self.elem_into_string(0, 0)
    }

    fn elem_into_string(&self, root: usize, tab_level: usize) -> String {
        let mut res = String::new();

        let element = &self.element_store[root];
        let full_name = gen_full_name(&element.prefix, &element.tag);

        for _ in 0..tab_level {
            res += "\t"
        }

        res += "<";
        res += &full_name;

        for attr in &element.attributes {
            res += " ";
            res += &gen_full_name(&attr.prefix, &attr.tag);
            if let Some(val) = &attr.value {
                res += "=\"";
                res += val;
                res += "\"";
            }
        }

        if element.children.is_empty() {
            res += " />\n"
        } else {
            res += ">\n";
            for (entity_type, child) in &element.children {
                match entity_type {
                    EntityType::Element => res += &self.elem_into_string(*child, tab_level + 1),
                    EntityType::Text => {
                        for _ in 0..tab_level + 1 {
                            res += "\t"
                        }
                        res += &self.text_store[*child];
                        res += "\n";
                    }
                };
            }

            for _ in 0..tab_level {
                res += "\t"
            }
            res += "</";
            res += &full_name;
            res += ">\n";
        }

        res
    }
}

fn validate_document_start(
    document_source: &str,
    first_token: Option<Result<Token, trax_parser::Error>>,
) -> Result<(), DocumentParseError> {
    match first_token {
        Some(Ok(Token::ElementStart { prefix, local, .. }))
            if local.as_str() == "document" && prefix.as_str().is_empty() =>
        {
            Ok(())
        }
        Some(Ok(token)) => Err(DocumentParseError::InvalidRootElement(r(
            document_source,
            token.span(),
        ))),
        Some(Err(e)) => Err(e.into()),
        None => Err(DocumentParseError::EmptyDocument),
    }
}

fn gen_full_name(prefix: &str, local: &str) -> String {
    if prefix.is_empty() {
        local.into()
    } else {
        format!("{prefix}:{local}")
    }
}
