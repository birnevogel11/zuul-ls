use std::path::Path;
use std::path::PathBuf;

use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionResponse, CompletionTriggerKind, Position,
};

use crate::ls::symbols::ZuulSymbol;

use super::parser::AutoCompleteToken;

pub fn get_trigger_char(context: Option<CompletionContext>) -> Option<String> {
    let context = context?;
    if context.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER {
        return context.trigger_character.clone();
    }
    None
}

pub fn complete_items(
    symbols: &ZuulSymbol,
    path: &Path,
    content: &Rope,
    position: &Position,
) -> Option<(CompletionResponse, Option<AutoCompleteToken>)> {
    None
}
