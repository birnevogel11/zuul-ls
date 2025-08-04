use tower_lsp::Server;

use zuul_parser::log::init_logging;
use zuul_parser::ls::server::initialize_service;

#[tokio::main]
async fn main() {
    let _ = init_logging();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = initialize_service();

    Server::new(stdin, stdout, socket).serve(service).await;
}
