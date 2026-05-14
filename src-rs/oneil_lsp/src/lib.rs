// TODO: remove the `allow`s once I have the chance to resolve the issues.
#![allow(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(dead_code)]

pub mod custom_requests;
mod definition;

mod diagnostics;
mod doc_store;
mod hover;
mod location;
mod path;
mod symbol_lookup;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use oneil_runtime::Runtime as OneilRuntime;
use oneil_shared::paths::{ModelPath, SourcePath};
use tower_lsp_server::ls_types::OneOf;
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{self, Result},
    ls_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, ExecuteCommandOptions, ExecuteCommandParams,
        GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability,
        InitializeParams, InitializeResult, InitializedParams, LSPAny, MessageType,
        PositionEncodingKind, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
        TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri,
    },
};

use definition::resolve_definition;
use diagnostics::diagnostics_from_runtime_errors;
use doc_store::DocumentStore;
use hover::hover_markdown;
use location::span_to_range;

#[tokio::main]
pub async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let docs = Arc::new(DocumentStore::new());
    let runtime = Mutex::new(OneilRuntime::new());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs,
        workspace_roots: Mutex::new(Vec::new()),
        runtime,
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

struct Backend {
    client: Client,
    docs: Arc<DocumentStore>,
    /// Workspace folder paths from `initialize`, longest first (nested folders match innermost).
    workspace_roots: Mutex<Vec<PathBuf>>,
    // TODO: figure out how to handle async runtime operations better.
    //
    //       Right now, only one thing can use the runtime at a time.
    runtime: Mutex<OneilRuntime>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "initialize called")
            .await;

        let workspace_roots = params
            .workspace_folders
            .as_ref()
            .map(|folders| {
                folders
                    .iter()
                    .filter_map(|folder| {
                        folder.uri.to_file_path().map(std::borrow::Cow::into_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();

        *self
            .workspace_roots
            .lock()
            .expect("workspace_roots mutex poisoned") = workspace_roots;

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
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        open_close: Some(true),
                        ..Default::default()
                    },
                )),
                position_encoding: Some(PositionEncodingKind::UTF16),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["oneil/instanceTree".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "oneil-lsp-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized called")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "shutdown called")
            .await;

        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did_open called")
            .await;

        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        self.docs.open(params.text_document).await;

        if let Ok(model_path) = ModelPath::try_from(uri.path().as_str()) {
            self.publish_diagnostics_for_model_path(&model_path, Some(version))
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did_change called")
            .await;

        let result = self
            .docs
            .apply_changes(params.text_document, params.content_changes)
            .await;

        if let Err(error) = result {
            self.client
                .log_message(MessageType::ERROR, format!("did_change error: {error}"))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did_close called")
            .await;

        self.docs.close(params.text_document).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did_save called")
            .await;

        let uri = params.text_document.uri.clone();

        if let Ok(model_path) = ModelPath::try_from(uri.path().as_str()) {
            // Invalidate derived caches (unit graph, eval) for the saved file
            // before re-evaluating. Without this, `build_unit_graph_inner`
            // returns the stale cached graph even though the source changed.
            {
                let source_path = SourcePath::from(&model_path);
                let mut runtime = self.runtime.lock().expect("runtime mutex poisoned");
                let _ = runtime.load_source(&source_path);
            }
            self.publish_diagnostics_for_model_path(&model_path, None)
                .await;
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.client
            .log_message(MessageType::INFO, "goto_definition called")
            .await;

        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;

        let Ok(current_model_path) = ModelPath::try_from(uri.path().as_str()) else {
            return Ok(None);
        };

        // Convert LSP position to byte offset
        let Some(offset) = self.docs.position_to_offset(&uri, position).await else {
            self.client
                .log_message(MessageType::WARNING, "Could not convert position to offset")
                .await;

            return Ok(None);
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "goto_definition: offset={}, position={}:{}",
                    offset, position.line, position.character
                ),
            )
            .await;

        // To avoid async problems with holding a mutex guard across an await,
        // we return a tuple of the result and maybe a log message.
        //
        // Each `break 'complete (result, maybe_log_message);` can be thought of as
        // `log(log_message); return result;`
        let (result, maybe_log_message) = 'complete: {
            let mut runtime = self
                .runtime
                .lock()
                .expect("if the runtime has panicked elsewhere, it is not in a useful state");

            let (ir_model, errors) = runtime.load_and_lower(&current_model_path);

            let Some(ir_model) = ir_model else {
                let errors = errors.to_vec();
                break 'complete (Ok(None), Some(format!("Error loading IR: {errors:?}")));
            };

            // Find the symbol at the cursor position
            let Some(symbol) = symbol_lookup::find_symbol_at_offset(ir_model, offset) else {
                break 'complete (Ok(None), Some("No symbol found at position".to_string()));
            };

            // Resolve the symbol to its definition location
            let location = resolve_definition(&symbol, &mut runtime, &current_model_path);

            let log_message =
                format!("Found symbol: {symbol:?}, definition location: {location:?}");

            (
                Ok(location.map(GotoDefinitionResponse::Scalar)),
                Some(log_message),
            )
        };

        if let Some(log_message) = maybe_log_message {
            self.client
                .log_message(MessageType::INFO, log_message)
                .await;
        }

        result
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        if params.command != "oneil/instanceTree" {
            return Ok(None);
        }

        let uri_str = params
            .arguments
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| jsonrpc::Error {
                code: jsonrpc::ErrorCode::InvalidParams,
                message: "oneil/instanceTree requires a file URI argument".into(),
                data: None,
            })?;

        // Parse the URI properly to handle file:// URLs across platforms
        let uri = uri_str.parse::<Uri>().map_err(|e| jsonrpc::Error {
            code: jsonrpc::ErrorCode::InvalidParams,
            message: format!("invalid URI: {uri_str:?} ({e})").into(),
            data: None,
        })?;
        let file_path = uri.to_file_path().ok_or_else(|| jsonrpc::Error {
            code: jsonrpc::ErrorCode::InvalidParams,
            message: format!("URI is not a file path: {uri_str:?}").into(),
            data: None,
        })?;

        let model_path = ModelPath::try_from(file_path.as_ref()).map_err(|()| jsonrpc::Error {
            code: jsonrpc::ErrorCode::InvalidParams,
            message: format!("invalid model path: {}", file_path.display()).into(),
            data: None,
        })?;

        let result = {
            let mut runtime = self.runtime.lock().expect("runtime mutex poisoned");
            custom_requests::build_instance_tree(&mut runtime, &model_path)
        };

        match result {
            Ok(tree) => {
                let json = serde_json::to_value(tree).map_err(|e| jsonrpc::Error {
                    code: jsonrpc::ErrorCode::InternalError,
                    message: format!("serialization error: {e}").into(),
                    data: None,
                })?;
                Ok(Some(json))
            }
            Err(msg) => Err(jsonrpc::Error {
                code: jsonrpc::ErrorCode::InternalError,
                message: msg.into(),
                data: None,
            }),
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;

        let Ok(current_model_path) = ModelPath::try_from(uri.path().as_str()) else {
            return Ok(None);
        };

        let Some(offset) = self.docs.position_to_offset(&uri, position).await else {
            return Ok(None);
        };

        let (result, maybe_log_message) = 'complete: {
            let mut runtime = self
                .runtime
                .lock()
                .expect("if the runtime has panicked elsewhere, it is not in a useful state");

            let (ir_model, errors) = runtime.load_and_lower(&current_model_path);

            let Some(ir_model) = ir_model else {
                break 'complete (
                    Ok(None),
                    Some(format!("hover: error loading IR: {errors:?}")),
                );
            };

            let Some(symbol) = symbol_lookup::find_symbol_at_offset(ir_model, offset) else {
                break 'complete (Ok(None), Some("hover: no symbol at position".to_string()));
            };

            let workspace_roots = self
                .workspace_roots
                .lock()
                .expect("workspace_roots mutex poisoned")
                .clone();

            let markdown =
                hover_markdown(&symbol, &mut runtime, &current_model_path, &workspace_roots);

            let has_contents = markdown.is_some();
            let hover_range = Some(span_to_range(&symbol.span()));

            let hover = markdown.map(|contents| Hover {
                contents,
                range: hover_range,
            });

            (
                Ok(hover),
                Some(format!(
                    "hover: symbol={symbol:?}, has_contents={has_contents}"
                )),
            )
        };

        if let Some(log_message) = maybe_log_message {
            self.client
                .log_message(MessageType::INFO, log_message)
                .await;
        }

        result
    }
}

impl Backend {
    /// Evaluates the model at the given URI and publishes any errors as LSP diagnostics.
    async fn publish_diagnostics_for_model_path(
        &self,
        model_path: &ModelPath,
        version: Option<i32>,
    ) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("publish_diagnostics_for_model_path: {model_path:?}, version: {version:?}"),
            )
            .await;

        let (successful_models, diagnostics) = {
            let mut runtime = self.runtime.lock().expect("runtime mutex poisoned");
            // Use the compose-only entry point: file / IR / composition
            // / validation diagnostics surface here, no eval pass.
            // The full evaluation path is reserved for explicit `oneil
            // eval`-style invocations; IDE feedback only needs static
            // diagnostics initially.
            //
            // When a .one design file is checked, the runtime automatically
            // redirects to the declared target model and applies the design,
            // so tests in the design file are validated in the target's scope.
            let (visited_paths, errors) = runtime.check_model(model_path);

            // Mirror the prior `result.all_model_paths()` clear set:
            // every file the composition touched gets its existing
            // diagnostics cleared, so stale errors on now-clean files
            // don't linger after the user fixes them. Files that
            // re-acquire diagnostics get them re-published below.
            // The visited_paths already includes design files since
            // check_model handles that internally.
            let successful_models = visited_paths
                .into_iter()
                .map(ModelPath::into_path_buf)
                .filter_map(Uri::from_file_path);

            let diagnostics = diagnostics_from_runtime_errors(&errors);

            (successful_models, diagnostics)
        };

        for uri in successful_models {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("clearing diagnostics for successful model: {uri:?}"),
                )
                .await;

            // clear diagnostics for successful models
            self.client
                .publish_diagnostics(uri.clone(), vec![], version)
                .await;
        }

        // publish new diagnostics
        for (uri, diagnostics) in diagnostics {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("publishing diagnostics for {uri:?}: {diagnostics:?}"),
                )
                .await;

            self.client
                .publish_diagnostics(uri, diagnostics, version)
                .await;

            self.client
                .log_message(MessageType::INFO, "diagnostics published".to_string())
                .await;
        }
    }
}
