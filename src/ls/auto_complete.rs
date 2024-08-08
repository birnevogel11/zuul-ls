use std::path::Path;

use phf::phf_map;
use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, Position,
};
use walkdir::WalkDir;

use super::go_to_definition::parse_local_vars_ansible;
use super::parser::{parse_token, AutoCompleteToken, TokenType};
use super::symbols::ZuulSymbol;

use crate::ls::variable_group::process_var_group;
use crate::parser::variable::{VariableGroup, VariableGroupInfo};
use crate::path::{retrieve_repo_path, shorten_path, to_path};

static ZUUL_PROPERTY: phf::Map<&'static str, &[&'static str]> = phf_map! {
    "job" => &["abstract", "description", "name", "nodeset", "parent",
               "post-run", "pre-run", "required-projects", "roles", "run", "vars", "voting", "secrets"],
    "project-template" => &["name", "queue"],
};

fn fill_guess_content(content: &Rope, position: &Position) -> Rope {
    let mut try_content = content.clone();
    let line_idx = position.line as usize;
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
    try_content
}

fn get_token(path: &Path, content: &Rope, position: &Position) -> Option<AutoCompleteToken> {
    parse_token(path, content, position).or_else(|| {
        let line_idx = position.line as usize;
        let line = content.get_line(line_idx)?;
        if line.to_string().contains(':') {
            None
        } else {
            let try_content = fill_guess_content(content, position);
            parse_token(path, &try_content, position)
        }
    })
}

fn render_variable_doc(variable_info: &VariableGroupInfo) -> String {
    variable_info
        .variable_locs
        .iter()
        .map(|vi| {
            format!(
                "{} ({:})\n",
                &vi.value,
                shorten_path(&vi.name.path).display()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn complete_variable_item_internal(
    value: &str,
    var_stack: &[String],
    var_group: &VariableGroup,
) -> Option<Vec<CompletionItem>> {
    process_var_group(value, var_stack, var_group, 0, |value, var_group| {
        let items = var_group
            .iter()
            .filter(|entry| entry.key().starts_with(value) && entry.key() != value)
            .map(|entry| CompletionItem {
                label: entry.key().to_string(),
                documentation: Some(Documentation::String(render_variable_doc(entry.value()))),
                kind: Some(CompletionItemKind::VARIABLE),
                ..CompletionItem::default()
            })
            .collect::<Vec<_>>();
        Some(items)
    })
}

fn complete_variable_items(
    token: &AutoCompleteToken,
    symbols: &ZuulSymbol,
    path: &Path,
    content: &Rope,
    position: &Position,
) -> Vec<CompletionItem> {
    if let TokenType::Variable(var_stack) = &token.token_type {
        let mut local_vars: VariableGroup = parse_local_vars_ansible(path, content, token);
        if local_vars.is_empty() {
            let try_content = fill_guess_content(content, position);
            local_vars = parse_local_vars_ansible(path, &try_content, token);
        }

        let var_stack = match var_stack {
            Some(var_stack) => var_stack,
            None => &Vec::new(),
        };

        [&local_vars, symbols.vars()]
            .into_iter()
            .flat_map(|vg| {
                complete_variable_item_internal(&token.value, var_stack, vg).unwrap_or_default()
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    }
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
        TokenType::Variable { .. } => Some((
            CompletionResponse::Array(complete_variable_items(
                &token, symbols, path, content, position,
            )),
            token,
        )),
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
    }
}
