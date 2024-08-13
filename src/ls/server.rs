use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService};

use super::auto_complete::complete_items;
use super::cache::AutoCompleteCache;
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
    auto_complete_cache: AutoCompleteCache,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let mut trigger_chars: Vec<char> = ('a'..='z').collect();
        ['/', '-', '_', '.']
            .iter()
            .for_each(|c| trigger_chars.push(*c));

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(
                        trigger_chars.into_iter().map(|c| c.to_string()).collect(),
                    ),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),

                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.initialize_zuul().await;
        log::debug!("client: {:#?}", self);
        self.client
            .log_message(MessageType::INFO, "zuul-ls initialized!")
            .await;
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

    async fn will_save(&self, _: WillSaveTextDocumentParams) {}

    async fn will_save_wait_until(
        &self,
        _: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        Ok(None)
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {}

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        log::info!("did change params: {:#?}", params);
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        log::info!("did save params: {:#?}", params);
        let uri = &params.text_document.uri;
        let path = uri.to_file_path().unwrap();

        log::info!("Update symbols");
        self.symbols.update(&path);

        log::info!("Clean auto complete cache");
        self.auto_complete_cache.clear();
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.on_go_to_definition(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.on_completion(params).await
    }

    async fn symbol(
        &self,
        _params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        // TODO: implement it
        Ok(None)
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        // TODO: implement it
        Ok(params)
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

        let content = self.document_map.get(&uri.to_string());
        let position = &params.text_document_position.position;
        let path = uri.to_file_path().unwrap();

        let response =
            match self
                .auto_complete_cache
                .get_cached_items(uri_path, position, &params.context)
            {
                Some((cached_response, value)) => {
                    log::info!("Cached auto-complete cache: {:#?}", value);
                    self.auto_complete_cache.add_entry(
                        uri.to_string(),
                        position,
                        &cached_response,
                        &value,
                    );
                    Some(cached_response)
                }
                None => content
                    .as_ref()
                    .and_then(|c| complete_items(&self.symbols, &path, c, position))
                    .map(|(response, token)| {
                        self.auto_complete_cache.add_entry(
                            uri.to_string(),
                            position,
                            &response,
                            &token.value,
                        );
                        response
                    }),
            };

        Ok(response)
    }
}

pub fn initialize_service() -> (tower_lsp::LspService<Backend>, tower_lsp::ClientSocket) {
    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        symbols: ZuulSymbol::default(),
        auto_complete_cache: AutoCompleteCache::default(),
    })
    .finish();

    (service, socket)
}
