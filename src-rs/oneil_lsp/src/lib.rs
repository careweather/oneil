#![allow(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
    Location, MarkedString, MessageType, Position, PositionEncodingKind, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server, UriExt};

mod doc_store;

use doc_store::DocumentStore;

#[derive(Debug)]
struct Backend {
    client: Client,
    docs: Arc<DocumentStore>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // let params_string = format!("{params:#?}");
        // self.client
        //     .log_message(MessageType::INFO, params_string)
        //     .await;

        let encodings_str = params
            .capabilities
            .general
            .and_then(|general| general.position_encodings)
            .map(|encodings| format!("encodings: {encodings:?}"))
            .unwrap_or_default();
        self.client
            .log_message(MessageType::INFO, encodings_str)
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // VS Code currently expects UTF-16 unless explicitly configured, so advertise UTF-16.
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        open_close: Some(true),
                        ..Default::default()
                    },
                )),
                position_encoding: Some(PositionEncodingKind::UTF16),
                definition_provider: Some(tower_lsp_server::lsp_types::OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "oneil-lsp-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;

        let params_string = format!("{params:#?}");
        self.client
            .log_message(MessageType::INFO, params_string)
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hovering time!")
            .await;

        let params_string = format!("{params:#?}");
        self.client
            .log_message(MessageType::INFO, params_string)
            .await;

        let position = params.text_document_position_params.position;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're *hovering*!".to_string())),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: position.character,
                },
                end: Position {
                    line: position.line,
                    character: position.character + 4,
                },
            }),
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "opened").await;

        let params_str = format!("{params:#?}");
        self.client.log_message(MessageType::INFO, params_str).await;

        self.docs.open(params.text_document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let result = self
            .docs
            .apply_changes(params.text_document, params.content_changes)
            .await;

        if let Err(error) = result {
            let _ = self
                .client
                .log_message(MessageType::ERROR, format!("did_change error: {error}"))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.docs.close(params.text_document).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "opened").await;

        let params_str = format!("{params:#?}");
        self.client.log_message(MessageType::INFO, params_str).await;
    }

    // text_document_position_params: TextDocumentPositionParams {
    //     text_document: TextDocumentIdentifier {
    //         uri: Uri(
    //             Uri {
    //                 scheme: Some(
    //                     "file",
    //                 ),
    //                 authority: Some(
    //                     Authority {
    //                         userinfo: None,
    //                         host: Host {
    //                             text: "",
    //                             data: RegName(
    //                                 "",
    //                             ),
    //                         },
    //                         port: None,
    //                     },
    //                 ),
    //                 path: "/home/pgattic/work/oneil/test/unit_error.on",
    //                 query: None,
    //                 fragment: None,
    //             },
    //         ),
    //     },
    //     position: Position {
    //         line: 4,
    //         character: 15,
    //     },

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let _position = params.text_document_position_params.position;

        // Look up position in code
        // Determine meaning of token
        // Find source
        // Return source Location

        // NOTE: For the `Position` type, lines and characters are both 0-based, don't include the
        // last character, and do include the last line
        if true {
            let dest_location = Location {
                uri: tower_lsp_server::lsp_types::Uri::from_file_path(
                    "/home/pgattic/work/oneil/test/unit_error.on",
                )
                .expect("SHOOT"),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 3,
                    },
                    end: Position {
                        line: 2,
                        character: 3,
                    },
                },
            };

            return Ok(Some(GotoDefinitionResponse::Scalar(dest_location)));
        }
        Ok(None)
    }
}

#[tokio::main]
pub async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let docs = Arc::new(DocumentStore::new());
    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: docs.clone(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
