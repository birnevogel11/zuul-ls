use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use zuul_parser::search::path::to_path;

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        todo!()
    }

    async fn initialized(&self, _: InitializedParams) {
        todo!()
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
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
        document_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
