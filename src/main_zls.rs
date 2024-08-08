use chrono::Local;
use petgraph::data::DataMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use zuul_parser::config::get_work_dir;
use zuul_parser::search::path::{get_role_repo_dirs, to_path};
use zuul_parser::search::roles::list_roles;

#[derive(Debug)]
struct Backend {
    client: Client,
    role_dirs: DashMap<String, PathBuf>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // todo!()
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                // TODO: implement it
                // text_document_sync: Some(TextDocumentSyncCapability::Kind(
                //     TextDocumentSyncKind::FULL,
                // )),
                text_document_sync: None,

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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let content = fs::read_to_string(uri.path()).unwrap();
        let ropey = ropey::Rope::from_str(content.as_str());
        for line in ropey.lines() {
            log::debug!("line: {:#?}", line);
        }

        log::error!("goto params: {:#?}", params);
        log::error!("uri: {:#?}", uri);

        Ok(None)
    }
}

impl Backend {
    async fn initialize_zuul(&self) {
        let work_dir = get_work_dir(None);
        let config_path = None;
        let repo_dirs = get_role_repo_dirs(&work_dir, config_path);
        let role_dirs: Vec<(String, PathBuf)> = list_roles(&repo_dirs);

        for (name, path) in role_dirs {
            self.role_dirs.insert(name, path);
        }
    }
}

fn init_logging() -> Option<env_logger::Builder> {
    let mut builder = env_logger::Builder::new();

    if let Ok(path) = env::var("ZUUL_LS_LOG_PATH") {
        let path = to_path(&path);
        let target = Box::new(File::create(path).expect("Can't create file"));
        builder
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Debug)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
        Some(builder)
    } else {
        env_logger::init();
        None
    }
}

#[tokio::main]
async fn main() {
    let _ = init_logging();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        role_dirs: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
