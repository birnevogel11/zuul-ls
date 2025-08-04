use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::path::to_path;

const ZUUL_LS_LOG_VAR_NAME: [&str; 2] = ["ZUUL_LS_LOG_PATH", "ZUUL_SEARCH_LOG_PATH"];

fn get_log_path() -> Option<PathBuf> {
    ZUUL_LS_LOG_VAR_NAME
        .into_iter()
        .filter_map(|x| {
            let var = env::var(x).ok()?;
            let path = to_path(&var);
            path.is_file().then_some(path)
        })
        .next()
}

pub fn init_logging() -> Option<env_logger::Builder> {
    let mut builder = env_logger::Builder::new();

    match get_log_path() {
        Some(path) => {
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
        }
        None => {
            env_logger::init();
            None
        }
    }
}
