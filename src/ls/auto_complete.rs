use std::path::Path;

use phf::phf_map;
use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse,
    CompletionTriggerKind, Documentation, Position,
};
use walkdir::WalkDir;

use crate::path::retrieve_repo_path;
use crate::path::to_path;

use super::parser::{parse_token, TokenType};
use super::symbols::ZuulSymbol;

use super::parser::AutoCompleteToken;

static ZUUL_PROPERTY: phf::Map<&'static str, &[&'static str]> = phf_map! {
    "job" => &["abstract", "description", "name", "nodeset", "parent",
               "post-run", "pre-run", "required-projects", "roles", "run", "vars", "voting", "secrets"],
    "project-template" => &["name", "queue"],
};

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
            let line = try_content.get_line(line_idx).unwrap();

            let insert_text = if line.to_string().trim().starts_with('-') {
                // Guess it's a list item
                " fake_value"
            } else {
                // Guess it's a dict item
                ": fake_value"
            };
            try_content.insert(
                try_content.line_to_char(line_idx) + position.character as usize,
                insert_text,
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
            let role_docs = symbols
                .role_docs()
                .iter()
                .filter(|entry| entry.key().starts_with(&token.value))
                .map(|entry| (entry.key().clone(), entry.value().clone()));

            Some((
                CompletionResponse::Array(
                    role_docs
                        .map(|(name, doc)| CompletionItem {
                            label: name,
                            documentation: doc.map(Documentation::String),
                            kind: Some(CompletionItemKind::FUNCTION),
                            ..CompletionItem::default()
                        })
                        .collect(),
                ),
                token,
            ))
        }
        TokenType::Job => {
            let jobs = symbols
                .jobs()
                .iter()
                .filter(|entry| entry.key().starts_with(&token.value))
                .map(|entry| (entry.key().clone()));

            Some((
                CompletionResponse::Array(
                    jobs.map(|name| CompletionItem {
                        label: name,
                        kind: Some(CompletionItemKind::CLASS),
                        ..CompletionItem::default()
                    })
                    .collect(),
                ),
                token,
            ))
        }
        TokenType::ZuulProperty(zuul_config_name) => {
            ZUUL_PROPERTY.get(zuul_config_name).map(|keys| {
                (
                    CompletionResponse::Array(
                        keys.iter()
                            .map(|name| {
                                let mut s = name.to_string();
                                s.push(':');
                                s
                            })
                            .map(|name| CompletionItem {
                                label: name,
                                kind: Some(CompletionItemKind::PROPERTY),
                                ..CompletionItem::default()
                            })
                            .collect(),
                    ),
                    token,
                )
            })
        }
        TokenType::Playbook => {
            let path = to_path(path.to_str().unwrap());
            retrieve_repo_path(&path).map(|repo_path| {
                let playbook_dir = repo_path.join("playbooks");
                let playbooks = WalkDir::new(playbook_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|x| x.file_name().to_str().unwrap().ends_with(".yaml"))
                    .map(|x| {
                        x.into_path()
                            .strip_prefix(&repo_path)
                            .unwrap()
                            .to_path_buf()
                    })
                    .filter(|x| x.to_str().unwrap().starts_with(&token.value))
                    .collect::<Vec<_>>();

                (
                    CompletionResponse::Array(
                        playbooks
                            .into_iter()
                            .map(|path| CompletionItem {
                                label: path.to_str().unwrap().to_string(),
                                kind: Some(CompletionItemKind::FILE),
                                ..CompletionItem::default()
                            })
                            .collect::<Vec<_>>(),
                    ),
                    token,
                )
            })
        }
        // TODO: implement it
        // TokenType::Variable => {}
        // TokenType::VariableWithPrefix(_) => {}
        _ => None,
    }
}
