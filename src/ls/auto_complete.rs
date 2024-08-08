use std::path::Path;
use std::path::PathBuf;

use ropey::Rope;
use tower_lsp::lsp_types::{CompletionResponse, Position};

use crate::ls::symbols::ZuulSymbol;

use super::parser::AutoCompleteToken;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
struct AutoCompleteTokenCache {
    path: PathBuf,
    position: Position,
}

pub fn complete_items(
    symbols: &ZuulSymbol,
    path: &Path,
    content: &Rope,
    position: &Position,
) -> Option<(CompletionResponse, Option<AutoCompleteToken>)> {
    None
}
