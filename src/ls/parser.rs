mod ansible;
mod key_stack;
mod utils;
mod zuul;

use std::path::{Path, PathBuf};

use ropey::Rope;
use tower_lsp::lsp_types::Position;

use crate::path::{retrieve_repo_path, to_path};
use ansible::{parse_token_ansible, parse_token_var};
use zuul::parse_token_zuul_config;

fn get_exist_path(path: PathBuf) -> Option<PathBuf> {
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub enum TokenFileType {
    #[default]
    Unknown,
    ZuulConfig,
    Playbooks,
    AnsibleRoleDefaults,
    AnsibleRoleTasks {
        defaults_path: Option<PathBuf>,
    },
    AnsibleRoleTemplates {
        tasks_path: Option<PathBuf>,
        defaults_path: Option<PathBuf>,
    },
}

impl TokenFileType {
    pub fn parse_path(path: &Path) -> Option<TokenFileType> {
        let path = to_path(path.to_str().unwrap());
        let repo_path = retrieve_repo_path(&path)?;

        let ls_file_type = [
            ("zuul.d", TokenFileType::ZuulConfig),
            ("playbooks", TokenFileType::Playbooks),
        ]
        .into_iter()
        .find_map(|(name, ls_path_type)| {
            let mut base_path = PathBuf::from(&repo_path);
            base_path.push(name);
            if path.starts_with(&base_path) {
                Some(ls_path_type)
            } else {
                None
            }
        });
        if ls_file_type.is_some() {
            return ls_file_type;
        }

        let mut base_path = PathBuf::from(&repo_path);
        base_path.push("roles");
        if path.starts_with(base_path) {
            return path
                .ancestors()
                .find_map(|ancestor| match ancestor.file_name() {
                    Some(name) => match name.to_str().unwrap() {
                        "defaults" => Some(TokenFileType::AnsibleRoleDefaults),
                        "tasks" => {
                            let role_dir = ancestor.parent()?;
                            let defaults_path =
                                get_exist_path(role_dir.join("defaults").join("main.yaml"));
                            Some(TokenFileType::AnsibleRoleTasks { defaults_path })
                        }
                        "templates" => {
                            let role_dir = ancestor.parent()?;
                            let defaults_path =
                                get_exist_path(role_dir.join("defaults").join("main.yaml"));
                            let tasks_path =
                                get_exist_path(role_dir.join("tasks").join("main.yaml"));
                            Some(TokenFileType::AnsibleRoleTemplates {
                                tasks_path,
                                defaults_path,
                            })
                        }
                        _ => None,
                    },
                    None => None,
                });
        }

        None
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub enum TokenSide {
    Left,
    #[default]
    Right,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub enum TokenType {
    #[default]
    Variable,
    VariableWithPrefix(Vec<String>),
    Role,
    Job,
    ZuulProperty(String),
    Playbook,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct AutoCompleteToken {
    pub value: String,
    pub file_type: TokenFileType,
    pub token_type: TokenType,
    pub token_side: TokenSide,
    pub key_stack: Vec<String>,
}

impl AutoCompleteToken {
    pub fn new_simple(value: String, file_type: TokenFileType, token_type: TokenType) -> Self {
        Self {
            value,
            file_type,
            token_type,
            ..Self::default()
        }
    }

    pub fn new(
        value: String,
        file_type: TokenFileType,
        token_type: TokenType,
        token_side: TokenSide,
        key_stack: Vec<String>,
    ) -> Self {
        Self {
            value,
            file_type,
            token_type,
            token_side,
            key_stack,
        }
    }
}

pub fn parse_token(path: &Path, content: &Rope, position: &Position) -> Option<AutoCompleteToken> {
    let file_type = TokenFileType::parse_path(path)?;

    match file_type {
        TokenFileType::ZuulConfig => parse_token_zuul_config(file_type, content, position),
        TokenFileType::Playbooks => parse_token_ansible(file_type, content, position),
        TokenFileType::AnsibleRoleTasks { .. } => parse_token_ansible(file_type, content, position),
        TokenFileType::AnsibleRoleDefaults { .. } => parse_token_var(file_type, content, position),
        TokenFileType::AnsibleRoleTemplates { .. } => parse_token_var(file_type, content, position),
        TokenFileType::Unknown => None,
    }
}
