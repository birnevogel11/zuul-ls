use std::path::Path;
use std::path::PathBuf;

use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse,
    CompletionTriggerKind, Position,
};

use super::parser::{parse_token, TokenType};
use super::symbols::ZuulSymbol;

use super::parser::AutoCompleteToken;

pub fn get_trigger_char(context: Option<CompletionContext>) -> Option<String> {
    let context = context?;
    if context.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER {
        return context.trigger_character.clone();
    }
    None
}

fn get_token(path: &Path, content: &Rope, position: &Position) -> Option<AutoCompleteToken> {
    parse_token(path, content, position).or_else(|| {
        let line_idx = position.line as usize;
        let line = content.get_line(line_idx)?;
        if line.to_string().contains(':') {
            None
        } else {
            let mut try_content = content.clone();
            try_content.insert(
                try_content.line_to_char(line_idx) + position.character as usize,
                ": fake_value",
            );
            parse_token(path, &try_content, position)
        }
    })
}

pub fn complete_items(
    symbols: &ZuulSymbol,
    path: &Path,
    content: &Rope,
    position: &Position,
) -> Option<(CompletionResponse, AutoCompleteToken)> {
    let token = get_token(path, content, position)?;
    log::info!("AutoCompleteToken: {:#?}", &token);

    match &token.token_type {
        TokenType::Role => {
            let mut role_names = symbols
                .role_dirs()
                .iter()
                .map(|entry| entry.key().clone())
                .filter(|x| x.starts_with(&token.value))
                .collect::<Vec<_>>();
            role_names.sort();

            return Some((
                CompletionResponse::Array(
                    role_names
                        .into_iter()
                        .map(|name| CompletionItem {
                            label: name.clone(),
                            insert_text: Some(name.clone()),
                            kind: Some(CompletionItemKind::FUNCTION),
                            sort_text: Some(name),
                            ..CompletionItem::default()
                        })
                        .collect(),
                ),
                token,
            ));
        }
        TokenType::Job => {}
        TokenType::Variable => {}
        TokenType::VariableWithPrefix(_) => {}
        TokenType::ZuulProperty(_) => {}
        TokenType::Playbook => {}
    };

    None
}
