/*!
Pull-based, zero-allocation TRAX parser.

## Example

```rust
for token in trax_parser::Tokenizer::from("<tagname name='value' modifier/>") {
    println!("{:?}", token);
}
```

## Differences from XML
- Removes CDATA, Processing Instructions, Entity Tokens, and DocType
- Adds modifiers - attributes without values
- Comments are formatted as /* ... */

## Benefits

- All tokens contain `StrSpan` structs which represent the position of the substring
  in the original document.
- Good error processing. All error types contain the position (line:column) where it occurred.
- No heap allocations.
- No dependencies.
- Tiny. ~1400 LOC and ~30KiB in the release build according to `cargo-bloat`.
- Only uses `core`

## Limitations

- No tree structure validation. `<root><child></root></child>`
  will be parsed without errors.
- Duplicated attributes is not an error. `<item a="v1" a="v2"/>`
  will be parsed without errors.

## Safety

- The library must not panic. Any panic is considered a critical bug
  and should be reported.
- The library forbids unsafe code.
*/

#![no_std]
#![feature(error_in_core)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[macro_use]
extern crate std;

macro_rules! matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => true,
            _ => false
        }
    }
}

mod display;
mod error;
mod stream;
mod strspan;
mod xmlchar;

pub use crate::error::*;
pub use crate::stream::*;
pub use crate::strspan::*;
pub use crate::xmlchar::*;

/// An XML token.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Token<'a> {
    /// Comment token.
    ///
    /// ```text
    /// /* text */
    ///    ----    - text
    /// ---------- - span
    /// ```
    Comment {
        text: StrSpan<'a>,
        span: StrSpan<'a>,
    },

    /// Element start token.
    ///
    /// ```text
    /// <ns:elem attr="value"/>
    ///  --                     - prefix
    ///     ----                - local
    /// --------                - span
    /// ```
    ElementStart {
        prefix: StrSpan<'a>,
        local: StrSpan<'a>,
        span: StrSpan<'a>,
    },

    /// Attribute token.
    ///
    /// ```text
    /// <elem ns:attr="value"/>
    ///       --              - prefix
    ///          ----         - local
    ///                -----  - value
    ///       --------------- - span
    /// ```
    Attribute {
        prefix: StrSpan<'a>,
        local: StrSpan<'a>,
        value: StrSpan<'a>,
        span: StrSpan<'a>,
    },

    /// Modifier token.
    ///
    /// ```text
    /// <elem ns:mod />
    ///       --              - prefix
    ///          ---          - local
    ///       ------          - span
    /// ```
    Modifier {
        prefix: StrSpan<'a>,
        local: StrSpan<'a>,
        span: StrSpan<'a>,
    },

    /// Element end token.
    ///
    /// ```text
    /// <ns:elem>text</ns:elem>
    ///                         - ElementEnd::Open
    ///         -               - span
    /// ```
    ///
    /// ```text
    /// <ns:elem>text</ns:elem>
    ///                -- ----  - ElementEnd::Close(prefix, local)
    ///              ---------- - span
    /// ```
    ///
    /// ```text
    /// <ns:elem/>
    ///                         - ElementEnd::Empty
    ///         --              - span
    /// ```
    ElementEnd {
        end: ElementEnd<'a>,
        span: StrSpan<'a>,
    },

    /// Text token.
    ///
    /// Contains text between elements including whitespaces.
    /// Basically everything between `>` and `<`.
    /// Except `]]>`, which is not allowed and will lead to an error.
    ///
    /// ```text
    /// <p> text </p>
    ///    ------     - text
    /// ```
    ///
    /// The token span is equal to the `text`.
    Text { text: StrSpan<'a> },
}

impl<'a> Token<'a> {
    /// Returns the [`StrSpan`] encompassing all of the token.
    pub fn span(&self) -> StrSpan<'a> {
        let span = match self {
            Token::Comment { span, .. } => span,
            Token::ElementStart { span, .. } => span,
            Token::Attribute { span, .. } => span,
            Token::Modifier { span, .. } => span,
            Token::ElementEnd { span, .. } => span,
            Token::Text { text, .. } => text,
        };
        *span
    }
}

/// `ElementEnd` token.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ElementEnd<'a> {
    /// Indicates `>`
    Open,
    /// Indicates `</name>`
    Close(StrSpan<'a>, StrSpan<'a>),
    /// Indicates `/>`
    Empty,
}

type Result<T> = core::result::Result<T, Error>;
type StreamResult<T> = core::result::Result<T, StreamError>;

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Root,
    Elements,
    Attributes,
    AfterElements,
    End,
}

/// Tokenizer for the XML structure.
#[derive(Clone)]
pub struct Tokenizer<'a> {
    stream: Stream<'a>,
    state: State,
    depth: usize,
    fragment_parsing: bool,
}

impl core::fmt::Debug for Tokenizer<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Tokenizer {{ ... }}")
    }
}

impl<'a> From<&'a str> for Tokenizer<'a> {
    #[inline]
    fn from(text: &'a str) -> Self {
        let mut stream = Stream::from(text);

        // Skip UTF-8 BOM.
        if stream.starts_with(&[0xEF, 0xBB, 0xBF]) {
            stream.advance(3);
        }

        stream.skip_spaces();

        Tokenizer {
            stream,
            state: State::Root,
            depth: 0,
            fragment_parsing: false,
        }
    }
}

macro_rules! map_err_at {
    ($fun:expr, $stream:expr, $err:ident) => {{
        let start = $stream.pos();
        $fun.map_err(|e| Error::$err(e, $stream.gen_text_pos_from(start)))
    }};
}

impl<'a> Tokenizer<'a> {
    /// Enables document fragment parsing.
    ///
    /// By default, `trax_parser` will check for DTD, root element, etc.
    /// But if we have to parse an XML fragment, it will lead to an error.
    /// This method switches the parser to the root element content parsing mode,
    /// so it will treat any data as a content of the root element.
    pub fn from_fragment(full_text: &'a str, fragment: core::ops::Range<usize>) -> Self {
        Tokenizer {
            stream: Stream::from_substr(full_text, fragment),
            state: State::Elements,
            depth: 0,
            fragment_parsing: true,
        }
    }

    fn parse_next_impl(&mut self) -> Option<Result<Token<'a>>> {
        let s = &mut self.stream;

        if s.at_end() {
            return None;
        }

        let start = s.pos();

        match self.state {
            State::Root => match s.curr_byte() {
                Ok(b'<') => match s.next_byte() {
                    Ok(b'/') | Err(_) => Some(Err(Error::InvalidElement(
                        StreamError::InvalidName,
                        s.gen_text_pos(),
                    ))),
                    Ok(_) => {
                        self.state = State::Attributes;
                        Some(Self::parse_element_start(s))
                    }
                },
                Ok(b'/') => match s.next_byte() {
                    Ok(b'*') => Some(Self::parse_comment(s)),
                    _ => Some(Err(Error::UnknownToken(s.gen_text_pos() - 1))),
                },
                _ => Some(Err(Error::UnknownToken(s.gen_text_pos()))),
            },
            State::Elements => {
                s.skip_spaces();

                // Use `match` only here, because only this section is performance-critical.
                match s.curr_byte() {
                    Ok(b'<') => match s.next_byte() {
                        Ok(b'/') => {
                            if self.depth > 0 {
                                self.depth -= 1;
                            }

                            if self.depth == 0 && !self.fragment_parsing {
                                self.state = State::AfterElements;
                            } else {
                                self.state = State::Elements;
                            }

                            Some(Self::parse_close_element(s))
                        }
                        Ok(_) => {
                            self.state = State::Attributes;
                            Some(Self::parse_element_start(s))
                        }
                        Err(_) => Some(Err(Error::UnknownToken(s.gen_text_pos()))),
                    },
                    Ok(b'/') => match s.next_byte() {
                        Ok(b'*') => Some(Self::parse_comment(s)),
                        _ => Some(Err(Error::UnknownToken(s.gen_text_pos() - 1))),
                    },
                    Ok(_) => Some(Self::parse_text(s)),
                    Err(_) => Some(Err(Error::UnknownToken(s.gen_text_pos()))),
                }
            }
            State::Attributes => {
                let t = Self::parse_attribute(s);

                if let Ok(Token::ElementEnd { end, .. }) = t {
                    if end == ElementEnd::Open {
                        self.depth += 1;
                    }

                    if self.depth == 0 && !self.fragment_parsing {
                        self.state = State::AfterElements;
                    } else {
                        self.state = State::Elements;
                    }
                }

                Some(t.map_err(|e| Error::InvalidAttribute(e, s.gen_text_pos_from(start))))
            }
            State::AfterElements => {
                if s.starts_with(b"/*") {
                    Some(Self::parse_comment(s))
                } else if s.starts_with_space() {
                    s.skip_spaces();
                    None
                } else {
                    Some(Err(Error::UnknownToken(s.gen_text_pos())))
                }
            }
            State::End => None,
        }
    }

    fn parse_comment(s: &mut Stream<'a>) -> Result<Token<'a>> {
        let start = s.pos();
        Self::parse_comment_impl(s)
            .map_err(|e| Error::InvalidComment(e, s.gen_text_pos_from(start)))
    }

    // '/*' ((Char - '-') | ('-' (Char - '-')))* '*/'
    fn parse_comment_impl(s: &mut Stream<'a>) -> StreamResult<Token<'a>> {
        let start = s.pos();
        s.advance(2);
        let text = s.consume_chars(|s, _| !(s.starts_with(b"*/")))?;
        s.skip_string(b"*/")?;

        let span = s.slice_back(start);

        Ok(Token::Comment { text, span })
    }

    fn parse_element_start(s: &mut Stream<'a>) -> Result<Token<'a>> {
        map_err_at!(Self::parse_element_start_impl(s), s, InvalidElement)
    }

    // '<' Name (S Attribute)* S? '>'
    fn parse_element_start_impl(s: &mut Stream<'a>) -> StreamResult<Token<'a>> {
        let start = s.pos();
        s.advance(1);
        let (prefix, local) = s.consume_qname()?;
        let span = s.slice_back(start);

        Ok(Token::ElementStart {
            prefix,
            local,
            span,
        })
    }

    fn parse_close_element(s: &mut Stream<'a>) -> Result<Token<'a>> {
        map_err_at!(Self::parse_close_element_impl(s), s, InvalidElement)
    }

    // '</' Name S? '>'
    fn parse_close_element_impl(s: &mut Stream<'a>) -> StreamResult<Token<'a>> {
        let start = s.pos();
        s.advance(2);

        let (prefix, tag_name) = s.consume_qname()?;
        s.skip_spaces();
        s.consume_byte(b'>')?;

        let span = s.slice_back(start);

        Ok(Token::ElementEnd {
            end: ElementEnd::Close(prefix, tag_name),
            span,
        })
    }

    // Name Eq AttValue
    fn parse_attribute(s: &mut Stream<'a>) -> StreamResult<Token<'a>> {
        let attr_start = s.pos();
        let has_space = s.starts_with_space();
        s.skip_spaces();

        if let Ok(c) = s.curr_byte() {
            let start = s.pos();

            match c {
                b'/' => {
                    s.advance(1);
                    s.consume_byte(b'>')?;
                    let span = s.slice_back(start);
                    return Ok(Token::ElementEnd {
                        end: ElementEnd::Empty,
                        span,
                    });
                }
                b'>' => {
                    s.advance(1);
                    let span = s.slice_back(start);
                    return Ok(Token::ElementEnd {
                        end: ElementEnd::Open,
                        span,
                    });
                }
                _ => {}
            }
        }

        if !has_space {
            if !s.at_end() {
                return Err(StreamError::InvalidSpace(
                    s.curr_byte_unchecked(),
                    s.gen_text_pos_from(attr_start),
                ));
            } else {
                return Err(StreamError::UnexpectedEndOfStream);
            }
        }

        let start = s.pos();

        let (prefix, local) = s.consume_qname()?;

        if s.try_consume_eq() {
            let quote = s.consume_quote()?;
            let quote_c = quote as char;
            // The attribute value must not contain the < character.
            let value = s.consume_chars(|_, c| c != quote_c && c != '<')?;
            s.consume_byte(quote)?;
            let span = s.slice_back(start);

            Ok(Token::Attribute {
                prefix,
                local,
                value,
                span,
            })
        } else {
            s.back();
            if !s.starts_with_space() {
                s.advance(1);
            }
            let span = s.slice_back(start);

            Ok(Token::Modifier {
                prefix,
                local,
                span,
            })
        }
    }

    fn parse_text(s: &mut Stream<'a>) -> Result<Token<'a>> {
        map_err_at!(Self::parse_text_impl(s), s, InvalidCharData)
    }

    fn parse_text_impl(s: &mut Stream<'a>) -> StreamResult<Token<'a>> {
        let start = s.pos();

        let mut chars = s.chars();
        loop {
            if let Some(c) = chars.next() {
                if !c.is_xml_char() {
                    return Err(StreamError::NonXmlChar(c, s.gen_text_pos()));
                } else if c == '/' {
                    if let Some(next) = chars.next() {
                        if next == '*' {
                            break;
                        }
                    }
                    s.advance(c.len_utf8());
                } else if c != '<' {
                    s.advance(c.len_utf8());
                } else {
                    break;
                }
            }
        }

        // trim trailing whitespace
        s.back();
        while s.starts_with_space() {
            s.back();
        }
        s.advance(1);

        let text = s.slice_back(start);

        Ok(Token::Text { text })
    }

    /// Returns a copy of the tokenizer's stream.
    pub fn stream(&self) -> Stream<'a> {
        self.stream
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut t = None;
        while !self.stream.at_end() && self.state != State::End && t.is_none() {
            t = self.parse_next_impl();
        }

        if let Some(Err(_)) = t {
            self.stream.jump_to_end();
            self.state = State::End;
        }

        t
    }
}
