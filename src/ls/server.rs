use std::path::PathBuf;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService};

use crate::config::get_work_dir;
use crate::ls::parser::{parse_word_type, AutoCompleteToken, Token};
use crate::parser::common::StringLoc;
use crate::parser::zuul::var_table::VariableInfo;
use crate::path::{get_role_repo_dirs, retrieve_repo_path, to_path};
use crate::search::jobs::list_job_locs_by_name;
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
    vars: DashMap<String, Vec<VariableInfo>>,
    jobs: DashMap<String, Vec<StringLoc>>,
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

    // TODO: implement it
    async fn did_save(&self, _params: DidSaveTextDocumentParams) {}

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
        vars.into_iter().for_each(|(name, var_info)| {
            self.vars.insert(name, var_info);
        });

        let jobs = list_job_locs_by_name(&work_dir, None);
        jobs.into_iter().for_each(|(name, job_locs)| {
            self.jobs.insert(name, job_locs);
        })
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
            if let Some(ac_word) = parse_word_type(uri, &content, position) {
                log::info!("AutoCompleteWord: {:#?}", &ac_word);
                return Ok(self.get_definition_list(uri.path(), &ac_word));
            }
        }

        Ok(None)
    }

    fn get_definition_list(
        &self,
        path: &str,
        ac_word: &AutoCompleteToken,
    ) -> Option<GotoDefinitionResponse> {
        let word = &ac_word.token;

        match ac_word.token_type {
            Token::Variable => {
                if let Some(var_infos) = self.vars.get(word) {
                    return Some(GotoDefinitionResponse::Array(
                        var_infos
                            .iter()
                            .map(|var| var.name.clone().into())
                            .collect::<Vec<_>>(),
                    ));
                }
            }
            Token::Job => {
                if let Some(job_locs) = self.jobs.get(word) {
                    return Some(GotoDefinitionResponse::Array(
                        job_locs
                            .iter()
                            .map(|job_loc| job_loc.clone().into())
                            .collect::<Vec<_>>(),
                    ));
                }
            }
            Token::Role => {
                if let Some(role) = self.role_dirs.get(word) {
                    return Some(GotoDefinitionResponse::Scalar(Location::new(
                        Url::from_file_path(role.value()).unwrap(),
                        Range::new(Position::new(0, 0), Position::new(0, 0)),
                    )));
                }
            }
            Token::ZuulProperty(_) => {}
            Token::Playbook => {
                let path = to_path(path);
                if let Some(repo_path) = retrieve_repo_path(&path) {
                    let playbook_path = repo_path.join(word);
                    if playbook_path.is_file() {
                        return Some(GotoDefinitionResponse::Scalar(Location::new(
                            Url::from_file_path(playbook_path).unwrap(),
                            Range::new(Position::new(0, 0), Position::new(0, 0)),
                        )));
                    }
                }
            }
        };

        None
    }
}

pub fn initialize_service() -> (tower_lsp::LspService<Backend>, tower_lsp::ClientSocket) {
    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        role_dirs: DashMap::new(),
        vars: DashMap::new(),
        jobs: DashMap::new(),
    })
    .finish();

    (service, socket)
}
