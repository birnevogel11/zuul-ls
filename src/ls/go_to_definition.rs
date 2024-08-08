use std::fs;
use std::path::Path;
use std::path::PathBuf;

use ropey::Rope;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::ls::parser::{AutoCompleteToken, TokenFileType, TokenType};
use crate::ls::symbols::ZuulSymbol;
use crate::parser::ansible::defaults::parse_defaults_vars;
use crate::parser::ansible::tasks::parse_task_vars;
use crate::parser::var_table::{merge_var_group, VarGroup};
use crate::path::{retrieve_repo_path, to_path};

use super::parser::parse_token;

fn append_ansible_vars<T>(
    var_group: VarGroup,
    path: &Option<PathBuf>,
    content: Option<String>,
    parse_fun: T,
) -> VarGroup
where
    T: Fn(&str, &Path, &str, &Path) -> Option<VarGroup>,
{
    let mut var_group = var_group;
    if let Some(path) = path {
        let content = content.unwrap_or(fs::read_to_string(path).unwrap_or_default());
        let ys = parse_fun(&content, path, "", &PathBuf::default());
        if let Some(ys) = ys {
            var_group = merge_var_group(var_group, ys);
        }
    }

    var_group
}

fn parse_local_vars_ansible(path: &Path, content: &Rope, token: &AutoCompleteToken) -> VarGroup {
    let mut var_group: VarGroup = VarGroup::new();
    match &token.file_type {
        TokenFileType::Playbooks => {}
        TokenFileType::AnsibleRoleDefaults => {
            var_group = append_ansible_vars(
                var_group,
                &Some(path.to_path_buf()),
                Some(content.to_string()),
                parse_defaults_vars,
            );
        }
        TokenFileType::AnsibleRoleTasks { defaults_path } => {
            var_group = append_ansible_vars(
                var_group,
                &Some(path.to_path_buf()),
                Some(content.to_string()),
                parse_task_vars,
            );
            var_group = append_ansible_vars(var_group, defaults_path, None, parse_defaults_vars);
        }
        TokenFileType::AnsibleRoleTemplates {
            tasks_path,
            defaults_path,
        } => {
            var_group = append_ansible_vars(var_group, defaults_path, None, parse_defaults_vars);
            var_group = append_ansible_vars(var_group, tasks_path, None, parse_task_vars);
        }
        _ => {}
    };

    var_group
}

fn find_var_definitions(
    value: &str,
    path: &Path,
    content: &Rope,
    symbols: &ZuulSymbol,
    token: &AutoCompleteToken,
) -> Option<GotoDefinitionResponse> {
    let mut var_info: Vec<Location> = Vec::new();

    let local_vars: VarGroup = parse_local_vars_ansible(path, content, token);
    if let Some(extra_var_info) = local_vars.get(value) {
        var_info.extend(
            extra_var_info
                .iter()
                .map(|var| var.name.clone().into())
                .collect::<Vec<_>>(),
        );
    }

    if let Some(extra_var_infos) = symbols.vars().get(value) {
        var_info.extend(
            extra_var_infos
                .iter()
                .map(|var| var.name.clone().into())
                .collect::<Vec<_>>(),
        );
    }

    if var_info.is_empty() {
        None
    } else if var_info.len() == 1 {
        Some(GotoDefinitionResponse::Scalar(var_info[0].clone()))
    } else {
        Some(GotoDefinitionResponse::Array(var_info))
    }
}

fn get_definition_list_internal(
    symbols: &ZuulSymbol,
    content: &Rope,
    path: &Path,
    token: &AutoCompleteToken,
) -> Option<GotoDefinitionResponse> {
    let value = &token.value;

    match &token.token_type {
        TokenType::Variable => {
            return find_var_definitions(value, path, content, symbols, token);
        }
        TokenType::VariableWithPrefix(var_stack) => {
            let mut var_name = var_stack.join(".");
            if !var_name.is_empty() {
                var_name.push('.');
            }
            var_name.push_str(&token.value);
            return find_var_definitions(&var_name, path, content, symbols, token);
        }
        TokenType::Job => {
            if let Some(job_locs) = symbols.jobs().get(value) {
                return Some(GotoDefinitionResponse::Array(
                    job_locs
                        .iter()
                        .map(|job_loc| job_loc.clone().into())
                        .collect::<Vec<_>>(),
                ));
            }
        }
        TokenType::Role => {
            if let Some(role) = symbols.role_dirs().get(value) {
                return Some(GotoDefinitionResponse::Scalar(Location::new(
                    Url::from_file_path(role.value()).unwrap(),
                    Range::new(Position::new(0, 0), Position::new(0, 0)),
                )));
            }
        }
        TokenType::ZuulProperty(_) => {}
        TokenType::Playbook => {
            let path = to_path(path.to_str().unwrap());
            if let Some(repo_path) = retrieve_repo_path(&path) {
                let playbook_path = repo_path.join(value);
                if playbook_path.is_file() {
                    return Some(GotoDefinitionResponse::Scalar(Location::new(
                        Url::from_file_path(playbook_path).unwrap(),
                        Range::new(Position::new(0, 0), Position::new(0, 0)),
                    )));
                }
            }
        }
    };

    None
}

pub fn get_definition_list(
    symbols: &ZuulSymbol,
    path: &Path,
    content: &Rope,
    position: &Position,
) -> Option<GotoDefinitionResponse> {
    let token = parse_token(path, content, position)?;
    log::info!("AutoCompleteWord: {:#?}", &token);
    get_definition_list_internal(symbols, content, path, &token)
}
