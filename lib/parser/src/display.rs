use core::fmt::Display;

use crate::{ElementEnd, Token, Tokenizer};

impl Display for Tokenizer<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut this = self.clone();
        let mut indent_level = 0;

        while let Some(Ok(token)) = this.next() {
            match token {
                Token::Comment { text, .. } => {
                    writeln!(f, "{:indent$}* {}", "", text, indent = indent_level * 4)?;
                }
                Token::ElementStart { prefix, local, .. } => {
                    writeln!(
                        f,
                        "{:indent$}< {}:{}",
                        "",
                        prefix,
                        local,
                        indent = indent_level * 4
                    )?;
                    indent_level += 1;
                }
                Token::ElementEnd { end, .. } => {
                    indent_level -= 1;
                    writeln!(f, "{:indent$}>", "", indent = indent_level * 4)?;
                    if end == ElementEnd::Open {
                        indent_level += 1;
                    }
                }
                Token::Attribute {
                    prefix,
                    local,
                    value,
                    ..
                } => {
                    writeln!(
                        f,
                        "{:indent$}- {}:{}=\"{}\"",
                        "",
                        prefix,
                        local,
                        value,
                        indent = indent_level * 4
                    )?;
                }
                Token::Modifier { prefix, local, .. } => {
                    writeln!(
                        f,
                        "{:indent$}+ {}:{}",
                        "",
                        prefix,
                        local,
                        indent = indent_level * 4
                    )?;
                }
                Token::Text { text } => {
                    writeln!(
                        f,
                        "{:indent$}{:?}",
                        "",
                        text.as_str(),
                        indent = indent_level * 4
                    )?;
                }
            }
        }
        Ok(())
    }
}
