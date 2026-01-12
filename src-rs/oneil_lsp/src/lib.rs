#![allow(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod doc_store;
mod symbol_lookup;

use oneil_runner::{builtins, file_parser};

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
    MarkedString, MessageType, Position, PositionEncodingKind, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

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
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;

        // Convert LSP position to byte offset
        let offset = match self.docs.position_to_offset(&uri, position).await {
            Some(offset) => offset,
            None => {
                self.client
                    .log_message(MessageType::WARNING, "Could not convert position to offset")
                    .await;
                return Ok(None);
            }
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!("goto_definition: offset={}, position={}:{}", offset, position.line, position.character),
            )
            .await;

        // Load and resolve the model
        let model_path = PathBuf::from(uri.path().as_str());
        let builtin_variables = builtins::Builtins::new();
        let model_collection = match oneil_model_resolver::load_model(
            &model_path,
            &builtin_variables,
            &file_parser::FileLoader,
        ) {
            Ok(collection) => collection,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "Failed to load model")
                    .await;
                return Ok(None);
            }
        };

        // Get the current model
        let current_model_path = oneil_ir::ModelPath::new(&model_path);
        let model = match model_collection.get_models().get(&current_model_path) {
            Some(model) => model,
            None => {
                self.client
                    .log_message(MessageType::ERROR, "Model not found in collection")
                    .await;
                return Ok(None);
            }
        };

        // Find the symbol at the cursor position
        let symbol = match symbol_lookup::find_symbol_at_offset(model, &current_model_path, offset)
        {
            Some(symbol) => {
                self.client
                    .log_message(MessageType::INFO, format!("Found symbol: {:?}", symbol))
                    .await;
                symbol
            }
            None => {
                // No symbol found at this position
                self.client
                    .log_message(MessageType::INFO, "No symbol found at position")
                    .await;
                return Ok(None);
            }
        };

        // Resolve the symbol to its definition location
        let location =
            symbol_lookup::resolve_definition(&symbol, &model_collection, &current_model_path);
        self.client.log_message(MessageType::INFO, format!("{:?}", symbol)).await;
        match &symbol {
            symbol_lookup::SymbolAtPosition::ParameterReference { name, .. } => {
                // Find the parameter in the current model
                let model = model_collection.get_models().get(&current_model_path);
                let param = model.map(|x| x.get_parameter(name));
                self.client.log_message(MessageType::INFO, format!("{:?}", model)).await;
                self.client.log_message(MessageType::INFO, format!("{:?}", param)).await;
            }
            _ => {}
        }

        Ok(location.map(GotoDefinitionResponse::Scalar))
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

