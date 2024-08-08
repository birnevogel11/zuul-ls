use chrono::Local;
use petgraph::data::DataMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
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

struct TextDocumentItem {
    uri: Url,
    text: String,
    version: i32,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
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
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
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

fn is_letter(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '/' | '-')
}

fn find_token(line: Rope, col: usize) -> Option<String> {
    let mut token_index = Vec::new();
    let mut b = -1;
    let mut e = -1;
    for (cidx, c) in line.chars().enumerate() {
        if !is_letter(c) {
            if b != -1 {
                e = cidx as i32;
                let ub = b as usize;
                let ue = e as usize;
                log::info!("b: {}, e: {}, token: {}", b, e, line.slice(ub..ue));
                token_index.push((ub, ue));

                b = -1;
                e = -1;
            }
        } else if is_letter(c) && b == -1 {
            b = cidx as i32;
        }
    }
    if b != -1 {
        let ub = b as usize;
        let ue = line.chars().len();
        log::info!("b: {}, e: {}, token: {}", b, e, line.slice(ub..ue));
        token_index.push((ub, ue));
    }
    for (b, e) in token_index {
        if col >= b && col <= e {
            let s = line.slice(b..e).to_string();
            log::info!("token: {:#?}", s);
            return Some(s);
        }
    }
    None
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }

    async fn initialize_zuul(&self) {
        let work_dir = get_work_dir(None);
        let config_path = None;
        let repo_dirs = get_role_repo_dirs(&work_dir, config_path);
        let role_dirs: Vec<(String, PathBuf)> = list_roles(&repo_dirs);

        for (name, path) in role_dirs {
            self.role_dirs.insert(name, path);
        }
    }

    async fn on_go_to_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let content = self.document_map.get(&uri.to_string()).unwrap();
        let line = content.line(params.text_document_position_params.position.line as usize);
        let col = params.text_document_position_params.position.character as usize;
        let token = find_token(line.into(), col);

        if let Some(name) = &token {
            let role = self.role_dirs.get(name);
            log::info!("role: {:#?}", role);
            if let Some(role) = role {
                let path = role.value();
                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    Url::from_file_path(path).unwrap(),
                    Range::new(Position::new(0, 0), Position::new(0, 0)),
                ))));
            }
        }

        log::info!("line: {:#?}, token: {:#?}", line, token);

        Ok(None)
    }
}

fn init_logging() -> Option<env_logger::Builder> {
    let mut builder = env_logger::Builder::new();

    if let Ok(path) = env::var("ZUUL_LS_LOG_PATH") {
        let ath = to_path(&path);
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
        document_map: DashMap::new(),
        role_dirs: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
