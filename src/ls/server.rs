use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService};

use super::auto_complete::complete_items;
use super::auto_complete::get_trigger_char;
use super::cache::AutoCompleteTokenCache;
use super::go_to_definition::get_definition_list;
use super::symbols::ZuulSymbol;

struct TextDocumentItem {
    uri: Url,
    text: String,
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
    symbols: ZuulSymbol,
    token_cache: DashMap<String, AutoCompleteTokenCache>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // todo!()
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                // TODO: implement it
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(
                        ('a'..='z')
                            .chain(std::iter::once('/'))
                            .chain(std::iter::once('.'))
                            .map(|x| x.to_string())
                            .collect(),
                    ),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                definition_provider: Some(OneOf::Left(true)),

                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.initialize_zuul().await;
        log::debug!("client: {:#?}", self);
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
        })
        .await
    }

    // TODO: implement it
    async fn did_save(&self, _params: DidSaveTextDocumentParams) {}

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.on_go_to_definition(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.on_completion(params).await
    }
}

impl Backend {
    async fn initialize_zuul(&self) {
        self.symbols.initialize();
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }

    async fn on_go_to_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let content = self.document_map.get(&uri.to_string());
        let position = &params.text_document_position_params.position;
        let path = uri.to_file_path().unwrap();

        Ok(content
            .as_ref()
            .and_then(|c| get_definition_list(&self.symbols, &path, c, position)))
    }

    async fn on_completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let uri_path = &uri.to_string();
        let position = &params.text_document_position.position;

        let content = self.document_map.get(&uri.to_string());
        let position = &params.text_document_position.position;
        let path = uri.to_file_path().unwrap();

        log::info!("params: {:#?}", params);
        Ok(content
            .as_ref()
            .and_then(|c| complete_items(&self.symbols, &path, c, position))
            .map(|(response, _)| response))
    }
}

pub fn initialize_service() -> (tower_lsp::LspService<Backend>, tower_lsp::ClientSocket) {
    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        symbols: ZuulSymbol::default(),
        token_cache: DashMap::new(),
    })
    .finish();

    (service, socket)
}
