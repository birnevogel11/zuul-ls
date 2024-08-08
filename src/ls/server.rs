use std::path::PathBuf;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService};

use crate::config::get_work_dir;
use crate::ls::parser::parse_current_word_type;
use crate::ls::parser::SearchType;
use crate::path::get_role_repo_dirs;
use crate::search::job_vars::VariableInfo;
use crate::search::roles::list_roles;
use crate::search::work_dir_vars::list_work_dir_vars_group;

struct TextDocumentItem {
    uri: Url,
    text: String,
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
    role_dirs: DashMap<String, PathBuf>,
    variables: DashMap<String, PathBuf>,
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
                // completion_provider: Some(CompletionOptions {
                //     resolve_provider: Some(false),
                //     work_done_progress_options: Default::default(),
                //     all_commit_characters: None,
                //     completion_item: None,
                // }),
                completion_provider: None,

                // TODO: implement it
                // references_provider: Some(OneOf::Left(true)),

                // Let's try to implement it first
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

    async fn did_save(&self, params: DidSaveTextDocumentParams) {}

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.on_go_to_definition(params).await
    }
}

impl Backend {
    async fn initialize_zuul(&self) {
        let work_dir = get_work_dir(None);
        let repo_dirs = get_role_repo_dirs(&work_dir, None);
        let role_dirs: Vec<(String, PathBuf)> = list_roles(&repo_dirs);

        role_dirs.into_iter().for_each(|(name, path)| {
            self.role_dirs.insert(name, path);
        });

        let vars = list_work_dir_vars_group(&work_dir, None);
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

        if let Some(content) = content {
            if let Some((current_word, search_types)) =
                parse_current_word_type(uri, &content, position)
            {
                log::info!("current_word: {:#?}", &current_word);
                log::info!("search_types: {:#?}", &search_types);
                return Ok(self.get_definition_list(&current_word, &search_types));
            }
        }

        Ok(None)
    }

    fn get_definition_list(
        &self,
        current_word: &str,
        search_types: &[SearchType],
    ) -> Option<GotoDefinitionResponse> {
        let mut locs: Vec<Location> = Vec::new();

        search_types.iter().for_each(|search_type| {
            match search_type {
                SearchType::Variable => todo!(), // TODO: implement it
                SearchType::Job => todo!(),      // TODO: implement it
                SearchType::Role => {
                    let role = self.role_dirs.get(current_word);
                    if let Some(role) = role {
                        let path = role.value();
                        locs.push(Location::new(
                            Url::from_file_path(path).unwrap(),
                            Range::new(Position::new(0, 0), Position::new(0, 0)),
                        ))
                    }
                }
            };
        });

        if locs.is_empty() {
            None
        } else if locs.len() == 1 {
            Some(GotoDefinitionResponse::Scalar(locs.remove(0)))
        } else {
            Some(GotoDefinitionResponse::Array(locs))
        }
    }
}

pub fn initialize_service() -> (tower_lsp::LspService<Backend>, tower_lsp::ClientSocket) {
    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        role_dirs: DashMap::new(),
        variables: DashMap::new(),
    })
    .finish();

    (service, socket)
}
