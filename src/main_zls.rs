use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;

use dashmap::DashMap;
use tower_lsp::Client;
use zuul_parser::search::path::to_path;

#[derive(Debug)]
struct Backend {
    client: Client,
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

fn main() {
    let _ = init_logging();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
}
