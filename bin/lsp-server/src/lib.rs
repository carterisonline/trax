use std::collections::BTreeMap;
use std::str::Split;

use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidDeleteFiles,
    DidOpenTextDocument, DidSaveTextDocument, Initialized, Notification,
};

use lsp_types::request::{Initialize, Request};
use lsp_types::{
    DeleteFilesParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, FileChangeType, FileDelete, FileEvent, InitializeResult,
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentItem, Url,
    VersionedTextDocumentIdentifier,
};

use serde_json::{from_value, Value};
use trax_parser::Token;

type Documents = BTreeMap<Url, TextDocumentItem>;

pub struct TraxLspServer<F: Fn(String)> {
    documents: Documents,
    log_fn: F,
}

impl<F: Fn(String)> TraxLspServer<F> {
    pub fn new(log_fn: F) -> Self {
        Self {
            documents: BTreeMap::new(),
            log_fn,
        }
    }

    pub fn on_notification(&mut self, method: &str, params: Value) -> Option<Value> {
        (self.log_fn)(format!("{method}, {params}"));
        match method {
            Initialize::METHOD => {
                return Some(
                    serde_json::to_value(InitializeResult {
                        capabilities: ServerCapabilities {
                            semantic_tokens_provider: Some(
                                SemanticTokensServerCapabilities::SemanticTokensOptions(
                                    SemanticTokensOptions {
                                        legend: SemanticTokensLegend {
                                            token_types: vec![
                                                SemanticTokenType::CLASS,
                                                SemanticTokenType::VARIABLE,
                                                SemanticTokenType::PROPERTY,
                                                SemanticTokenType::METHOD,
                                                SemanticTokenType::COMMENT,
                                            ],
                                            token_modifiers: vec![
                                                SemanticTokenModifier::DECLARATION,
                                            ],
                                        },
                                        range: Some(false),
                                        full: Some(SemanticTokensFullOptions::Bool(true)),
                                        ..Default::default()
                                    },
                                ),
                            ),
                            ..Default::default()
                        },
                        server_info: None,
                    })
                    .unwrap(),
                )
            }
            DidOpenTextDocument::METHOD => {
                let DidOpenTextDocumentParams { text_document } = from_value(params).unwrap();
                return self.on_did_open_text_document(text_document);
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = from_value(params).unwrap();

                // Ensure we receive full -- not incremental -- updates.
                assert_eq!(params.content_changes.len(), 1);
                let change = params.content_changes.into_iter().next().unwrap();
                assert!(change.range.is_none());

                let VersionedTextDocumentIdentifier { uri, version } = params.text_document;
                let updated_doc = TextDocumentItem::new(uri, "trax".into(), version, change.text);
                self.on_did_change_text_document(updated_doc);
            }
            DidChangeWatchedFiles::METHOD => {
                let DidChangeWatchedFilesParams { changes } = from_value(params).unwrap();
                let uris = changes.into_iter().map(|FileEvent { uri, typ }| {
                    assert_eq!(typ, FileChangeType::DELETED); // We only watch for `Deleted` events.
                    uri
                });
                self.on_did_delete_files(uris.collect());
            }
            DidDeleteFiles::METHOD => {
                let DeleteFilesParams { files } = from_value(params).unwrap();
                let mut uris = vec![];
                for FileDelete { uri } in files {
                    match Url::parse(&uri) {
                        Ok(uri) => uris.push(uri),
                        Err(e) => (self.log_fn)(format!("Failed to parse URI: {}", e)),
                    }
                }
                self.on_did_delete_files(uris);
            }
            // We don't care when a document is saved -- we already have the updated state thanks
            // to `DidChangeTextDocument`.
            DidSaveTextDocument::METHOD => (),
            // We don't care when a document is closed -- we care about all Polar files in a
            // workspace folder regardless of which ones remain open.
            DidCloseTextDocument::METHOD => (),

            // Nothing to do when we receive the `Initialized` notification.
            Initialized::METHOD => (),
            _ => (self.log_fn)(format!("on_notification {} {:?}", method, params)),
        }

        None
    }

    fn on_did_open_text_document(&mut self, doc: TextDocumentItem) -> Option<Value> {
        if let Some(TextDocumentItem { uri, text, .. }) = self.upsert_document(doc) {
            (self.log_fn)(text.clone());
            (self.log_fn)(format!("reopened tracked doc: {}", uri));

            let mut tokens = Vec::<SemanticToken>::new();

            for token in trax_parser::Tokenizer::from(text.as_str()) {
                match token {
                    Ok(Token::Attribute {
                        prefix,
                        local,
                        value,
                        span,
                    }) => tokens.push(SemanticToken {
                        delta_line: row(&text, local.end()),
                        delta_start: col(&text, local.end()),
                        length: local.len() as u32,
                        token_type: 2,
                        token_modifiers_bitset: 0,
                    }),
                    Ok(Token::Comment { text, span }) => (),
                    Ok(Token::ElementEnd { end, span }) => (),
                    Ok(Token::ElementStart {
                        prefix,
                        local,
                        span,
                    }) => (),
                    Ok(Token::Modifier {
                        prefix,
                        local,
                        span,
                    }) => (),
                    Ok(Token::Text { text }) => (),
                    Err(_) => break,
                }
            }

            return Some(
                serde_json::to_value(SemanticTokens {
                    result_id: None,
                    data: tokens,
                })
                .unwrap(),
            );
        }

        None
    }

    fn on_did_change_text_document(&mut self, doc: TextDocumentItem) {
        let uri = doc.uri.clone();
        if self.upsert_document(doc).is_none() {
            (self.log_fn)(format!("updated untracked doc: {}", uri));
        }
    }

    fn on_did_delete_files(&mut self, uris: Vec<Url>) {
        for uri in uris {
            let mut msg = format!("deleting URI: {}", uri);

            if self.remove_document(&uri).is_none() {
                msg += "\n\tchecking if URI is dir";
                let removed = self.remove_documents_in_dir(&uri);
                if removed.is_empty() {
                    if uri.as_str().ends_with(".trax") {
                        msg += "\n\tcannot remove untracked doc";
                    }
                } else {
                    for (uri, _) in removed {
                        msg += &format!("\n\t\tremoving dir member: {}", uri);
                    }
                }
            }
            (self.log_fn)(msg);
        }
    }

    fn upsert_document(&mut self, doc: TextDocumentItem) -> Option<TextDocumentItem> {
        self.documents.insert(doc.uri.clone(), doc)
    }

    fn remove_document(&mut self, uri: &Url) -> Option<TextDocumentItem> {
        self.documents.remove(uri)
    }

    fn remove_documents_in_dir(&mut self, dir: &Url) -> BTreeMap<Url, TextDocumentItem> {
        let (in_dir, not_in_dir): (Documents, Documents) =
            self.documents.clone().into_iter().partition(|(uri, _)| {
                let maybe_segments = dir.path_segments().zip(uri.path_segments());
                let compare_paths = |(l, r): (Split<_>, Split<_>)| l.zip(r).all(|(l, r)| l == r);
                maybe_segments.map_or(false, compare_paths)
            });

        self.documents = not_in_dir;
        in_dir
    }
}

fn row(text: &str, end: usize) -> u32 {
    let mut row = 1;
    for c in &text.as_bytes()[..end] {
        if *c == b'\n' {
            row += 1;
        }
    }

    row
}

fn col(text: &str, end: usize) -> u32 {
    let mut col = 1;
    for c in text[..end].chars().rev() {
        if c == '\n' {
            break;
        } else {
            col += 1;
        }
    }

    col
}
