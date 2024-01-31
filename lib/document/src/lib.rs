/*!
TRAX document parsing, rendering, and manipulation API.

## Example

```rust
let src = r#"<document>
    <one key="value" />
    <two with:modifier>
        <three />
    </two>
</document>"#;

let document = trax_document::Document::new(src).unwrap();

println!("{document:#?}");
```

## Safety

- The library must not panic. Any panic is considered a critical bug
  and should be reported.
- The library forbids unsafe code.
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{borrow::Cow, collections::VecDeque, fmt::Display};

use trax_parser::{span_text_range as r, ElementEnd, TextRange, Token, Tokenizer};

mod manipulation;

/// A reference to an entity in one of the [`Document`] stores.
#[derive(Debug, PartialEq, Clone)]
pub enum EntityRef {
    /// A refrence to an element in the element store.
    Element(usize),
    /// A reference to text in the text store.
    Text(usize),
}

impl Display for EntityRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityRef::Element(i) => write!(f, "Element at {i}"),
            EntityRef::Text(i) => write!(f, "Text at {i}"),
        }
    }
}

/// A TRAX attribute/modifier.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attribute<'a> {
    prefix: Cow<'a, str>,
    local: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
}

impl<'a> Attribute<'a> {
    /// Creates a new Attribute from its raw parts without allocating.
    pub fn new<C: Into<Cow<'a, str>>, C2: Into<Cow<'a, str>>, C3: Into<Cow<'a, str>>>(
        prefix: C,
        local: C2,
        value: Option<C3>,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            local: local.into(),
            value: value.map(|v| v.into()),
        }
    }
}

/// A TRAX element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element<'a> {
    parent: usize,
    children: VecDeque<EntityRef>,
    prefix: Cow<'a, str>,
    local: Cow<'a, str>,
    attributes: VecDeque<Attribute<'a>>,
}

/// A segment of text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Text<'a> {
    parent: usize,
    content: Cow<'a, str>,
}

/// A TRAX document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document<'a> {
    element_store: Vec<Option<Element<'a>>>,
    text_store: Vec<Option<Text<'a>>>,
}

/// An error encountered when parsing/creating a [`Document`].
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DocumentParseError {
    /// An error when processing an empty document. TRAX documents require a single root `<document>` element.
    #[error("Can't parse an empty document")]
    EmptyDocument,

    /// An error when the root `<document>` element is invalid.
    #[error("at {0}: Document must have a single root <document> element")]
    InvalidRootElement(TextRange),

    /// A generic syntax error. See [`trax_parser::Error`].
    #[error("syntax error")]
    SyntaxError(#[from] trax_parser::Error),

    /// An error when encountering an invalid tree structure (opening/closing elements out of order).
    #[error("invalid tree structure: `{closed_elem}` was closed ({location}) here but a different element `{current_open_elem}` was opened after it, and should be closed first")]
    InvalidTreeStructure {
        /// The element attempted to be closed.
        closed_elem: String,
        /// The current open element which should've been closed first.
        current_open_elem: String,
        /// The location of the attempted closed element.
        location: TextRange,
    },
}

impl<'a> Document<'a> {
    /// Create a new document.
    pub fn new(source: &'a str) -> Result<Self, DocumentParseError> {
        let mut tokenizer = Tokenizer::from(source);

        validate_document_start(source, tokenizer.next())?;

        let mut element_num = 1;
        let mut text_num = 0;
        let mut hierarchy = vec![0];
        let mut text_store = Vec::new();
        let mut element_store = vec![Some(Element {
            local: Cow::Borrowed("document"),
            ..Default::default()
        })];

        for token in tokenizer {
            let token = token?;
            let top_elem = *hierarchy.last().unwrap();

            match token {
                Token::ElementEnd {
                    end: ElementEnd::Empty,
                    ..
                } => {
                    hierarchy.pop();
                }

                Token::ElementStart { prefix, local, .. } => {
                    element_store[top_elem]
                        .as_mut()
                        .unwrap()
                        .children
                        .push_back(EntityRef::Element(element_num));

                    hierarchy.push(element_num);
                    element_store.push(Some(Element {
                        parent: top_elem,
                        prefix: Cow::Borrowed(prefix.as_str()),
                        local: Cow::Borrowed(local.as_str()),
                        ..Default::default()
                    }));

                    element_num += 1;
                }

                Token::Attribute {
                    prefix,
                    local,
                    value,
                    ..
                } => element_store[top_elem]
                    .as_mut()
                    .unwrap()
                    .attributes
                    .push_back(Attribute::new(
                        prefix.as_str(),
                        local.as_str(),
                        Some(value.as_str()),
                    )),

                Token::Modifier { prefix, local, .. } => element_store[top_elem]
                    .as_mut()
                    .unwrap()
                    .attributes
                    .push_back(Attribute::new(
                        prefix.as_str(),
                        local.as_str(),
                        None::<&str>,
                    )),

                // Ending the *current* open element
                Token::ElementEnd {
                    end: ElementEnd::Close(prefix, local),
                    ..
                } if element_store[top_elem].as_ref().unwrap().prefix == prefix.as_str()
                    && element_store[top_elem].as_ref().unwrap().local == local.as_str() =>
                {
                    hierarchy.pop();
                }

                // Closing an open element different from the current one makes an invalid tree
                // structure. Needed since the alloc-free parser doesn't do this.
                Token::ElementEnd {
                    end: ElementEnd::Close(prefix, local),
                    span,
                } => {
                    let current_open_elem = &element_store[top_elem];
                    return Err(DocumentParseError::InvalidTreeStructure {
                        closed_elem: gen_full_name(prefix.as_str(), &local),
                        current_open_elem: gen_full_name(
                            &current_open_elem.as_ref().unwrap().prefix,
                            &current_open_elem.as_ref().unwrap().local,
                        ),
                        location: r(source, span),
                    });
                }

                Token::Text { text } => {
                    element_store[top_elem]
                        .as_mut()
                        .unwrap()
                        .children
                        .push_back(EntityRef::Text(text_num));

                    text_store.push(Some(Text {
                        parent: top_elem,
                        content: Cow::Borrowed(text.as_str()),
                    }));

                    text_num += 1;
                }

                _ => (),
            }
        }

        Ok(Self {
            element_store,
            text_store,
        })
    }

    /// Render the document to plaintext.
    pub fn into_string(&self) -> String {
        self.elem_into_string(0, 0)
    }

    fn elem_into_string(&self, root: usize, tab_level: usize) -> String {
        let mut res = String::new();

        let element = &self.element_store[root].as_ref().unwrap();
        let full_name = gen_full_name(&element.prefix, &element.local);

        for _ in 0..tab_level {
            res += "\t"
        }

        res += "<";
        res += &full_name;

        for attr in &element.attributes {
            res += " ";
            res += &gen_full_name(&attr.prefix, &attr.local);
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
            for entity_ref in &element.children {
                match entity_ref {
                    EntityRef::Element(child) => {
                        res += &self.elem_into_string(*child, tab_level + 1)
                    }
                    EntityRef::Text(child) => {
                        for _ in 0..tab_level + 1 {
                            res += "\t"
                        }
                        res += &self.text_store[*child].as_ref().unwrap().content;
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
    if !prefix.is_empty() {
        format!("{prefix}:{local}")
    } else {
        local.to_string()
    }
}
