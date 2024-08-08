use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;

use zuul_parser::search::path::to_path;

fn init_logging(builder: &mut env_logger::Builder, raw_path: &str) {
    let path = to_path(raw_path);
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
}

fn main() {
    let mut builder = env_logger::Builder::new();

    if let Ok(path) = env::var("ZUUL_LS_LOG_PATH") {
        init_logging(&mut builder, &path);
    } else {
        env_logger::init();
    }

    log::error!("error");
    log::info!("info");
    log::info!("debug");
}
