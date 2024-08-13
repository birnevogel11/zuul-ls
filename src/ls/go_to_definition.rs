use std::fs;
use std::path::Path;
use std::path::PathBuf;

use ropey::Rope;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::ls::parser::{AutoCompleteToken, TokenFileType, TokenType};
use crate::ls::symbols::AnsibleRolePath;
use crate::ls::symbols::ZuulSymbol;
use crate::ls::variable_group::process_var_group;
use crate::parser::ansible::defaults::parse_defaults_vars;
use crate::parser::ansible::playbook::parse_playbook_vars;
use crate::parser::ansible::tasks::parse_task_vars;
use crate::parser::variable::VariableGroup;
use crate::path::{retrieve_repo_path, to_path};

use super::parser::parse_token;

fn parse_ansible_vars<T>(
    path: &Option<PathBuf>,
    content: Option<String>,
    parse_fun: T,
) -> VariableGroup
where
    T: Fn(&str, &Path, &str, &Path) -> Option<VariableGroup>,
{
    path.as_ref().map_or(VariableGroup::default(), |p| {
        let content = content.unwrap_or(fs::read_to_string(p).unwrap_or_default());
        parse_fun(&content, p, "", &PathBuf::default()).unwrap_or_default()
    })
}

pub fn parse_local_vars_ansible(
    path: &Path,
    content: &Rope,
    token: &AutoCompleteToken,
) -> VariableGroup {
    match &token.file_type {
        TokenFileType::Playbooks => parse_ansible_vars(
            &Some(path.to_path_buf()),
            Some(content.to_string()),
            parse_playbook_vars,
        ),
        TokenFileType::AnsibleRoleDefaults => parse_ansible_vars(
            &Some(path.to_path_buf()),
            Some(content.to_string()),
            parse_defaults_vars,
        ),
        TokenFileType::AnsibleRoleTasks { defaults_path } => {
            let mut xs = parse_ansible_vars(
                &Some(path.to_path_buf()),
                Some(content.to_string()),
                parse_task_vars,
            );
            let ys = parse_ansible_vars(defaults_path, None, parse_defaults_vars);

            xs.merge(ys);
            xs
        }
        TokenFileType::AnsibleRoleTemplates {
            tasks_path,
            defaults_path,
        } => {
            let mut xs = parse_ansible_vars(defaults_path, None, parse_defaults_vars);
            let ys = parse_ansible_vars(tasks_path, None, parse_task_vars);

            xs.merge(ys);
            xs
        }
        _ => VariableGroup::default(),
    }
}

pub fn parse_ansible_role_vars(ansible_path: &AnsibleRolePath) -> VariableGroup {
    let mut xs = parse_ansible_vars(&ansible_path.tasks_path, None, parse_task_vars);
    xs.merge(parse_ansible_vars(
        &ansible_path.defaults_path,
        None,
        parse_defaults_vars,
    ));

    xs
}

fn find_var_definitions_internal(
    value: &str,
    var_stack: &[String],
    var_group: &VariableGroup,
) -> Option<Vec<Location>> {
    process_var_group(value, var_stack, var_group, 0, |value, var_group| {
        let entry = var_group.get(value)?;
        Some(
            entry
                .value()
                .variable_locs
                .iter()
                .map(|vi| vi.name.clone().into())
                .collect(),
        )
    })
}

fn find_var_definitions(
    value: &str,
    var_stack: &Option<Vec<String>>,
    path: &Path,
    content: &Rope,
    symbols: &ZuulSymbol,
    token: &AutoCompleteToken,
) -> Option<GotoDefinitionResponse> {
    let local_vars: VariableGroup = parse_local_vars_ansible(path, content, token);

    let var_stack = match var_stack {
        Some(var_stack) => var_stack,
        None => &Vec::new(),
    };

    let var_info = [&local_vars, symbols.vars()]
        .into_iter()
        .flat_map(|vg| find_var_definitions_internal(value, var_stack, vg).unwrap_or_default())
        .collect::<Vec<_>>();

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
        TokenType::Variable { var_stack, .. } => {
            return find_var_definitions(value, var_stack, path, content, symbols, token);
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
        _ => {}
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
    log::info!("Go to definition - AutoCompleteToken: {:#?}", &token);
    get_definition_list_internal(symbols, content, path, &token)
}
