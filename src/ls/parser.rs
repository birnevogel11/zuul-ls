mod ansible;
mod key_stack;
mod utils;
mod zuul;

use std::path::{Path, PathBuf};

use ropey::Rope;
use tower_lsp::lsp_types::Position;
use yaml_rust2::Yaml;

use self::ansible::parse_token_ansible;
use self::key_stack::parse_value;
use self::utils::find_var_token;
use self::zuul::parse_token_zuul_config;
use crate::path::{retrieve_repo_path, to_path};

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AnsibleRolePath {
    pub tasks_path: Option<PathBuf>,
    pub defaults_path: Option<PathBuf>,
}

impl AnsibleRolePath {
    pub fn new(role_dir: &Path) -> Self {
        Self::_new(role_dir, true, true)
    }

    pub fn new_defaults_path(role_dir: &Path) -> Self {
        Self::_new(role_dir, false, true)
    }

    pub fn _new(role_dir: &Path, is_tasks_path: bool, is_defaults_path: bool) -> Self {
        let role_dir = role_dir.to_path_buf();
        let task_path = role_dir.join("tasks").join("main.yaml");
        let default_path = role_dir.join("defaults").join("main.yaml");

        Self {
            tasks_path: if is_tasks_path {
                task_path.is_file().then_some(task_path)
            } else {
                None
            },
            defaults_path: if is_defaults_path {
                default_path.is_file().then_some(default_path)
            } else {
                None
            },
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub enum TokenFileType {
    #[default]
    Unknown,
    ZuulConfig,
    Playbooks,
    AnsibleRoleDefaults,
    AnsibleRoleTasks(AnsibleRolePath),
    AnsibleRoleTemplates(AnsibleRolePath),
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
            path.starts_with(&base_path).then_some(ls_path_type)
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
                        "tasks" => Some(TokenFileType::AnsibleRoleTasks(
                            AnsibleRolePath::new_defaults_path(ancestor.parent()?),
                        )),
                        "templates" => Some(TokenFileType::AnsibleRoleTemplates(
                            AnsibleRolePath::new(ancestor.parent()?),
                        )),
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum TokenType {
    Variable {
        var_stack: Option<Vec<String>>,
        role_name: Option<String>,
    },
    Role,
    Job,
    ZuulProperty(String),
    Playbook,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Variable {
            var_stack: None,
            role_name: None,
        }
    }
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
        TokenFileType::Playbooks
        | TokenFileType::AnsibleRoleTasks { .. }
        | TokenFileType::AnsibleRoleDefaults { .. }
        | TokenFileType::AnsibleRoleTemplates { .. } => {
            parse_token_ansible(file_type, content, position)
        }
        TokenFileType::ZuulConfig => parse_token_zuul_config(file_type, content, position),
        TokenFileType::Unknown => None,
    }
}

pub struct VariableTokenBuilder(AutoCompleteToken);

impl VariableTokenBuilder {
    pub fn new(
        var_stack: Option<Vec<String>>,
        token_side: TokenSide,
        content: &Rope,
        position: &Position,
    ) -> Option<Self> {
        Self::_new_impl(var_stack, token_side, content, position, None)
    }

    pub fn new_with_role(
        var_stack: Option<Vec<String>>,
        token_side: TokenSide,
        content: &Rope,
        position: &Position,
        role_name: &Option<String>,
    ) -> Option<Self> {
        Self::_new_impl(var_stack, token_side, content, position, role_name.clone())
    }

    pub fn new_yaml(value: &Yaml, content: &Rope, position: &Position) -> Option<Self> {
        let (var_stack, token_side) = parse_value(value, None)?;
        Self::new(Some(var_stack), token_side, content, position)
    }

    fn _new_impl(
        var_stack: Option<Vec<String>>,
        token_side: TokenSide,
        content: &Rope,
        position: &Position,
        role_name: Option<String>,
    ) -> Option<Self> {
        let mut var_tokens = find_var_token(content, position)?;
        let token = var_tokens.pop()?;

        let var_stack = match token_side {
            TokenSide::Left => {
                let mut var_stack = var_stack.unwrap_or_default();
                var_stack.extend(var_tokens);
                var_stack
            }
            TokenSide::Right => var_tokens,
        };

        Some(VariableTokenBuilder(AutoCompleteToken {
            value: token,
            token_type: TokenType::Variable {
                var_stack: (!var_stack.is_empty()).then_some(var_stack),
                role_name,
            },
            token_side,
            ..AutoCompleteToken::default()
        }))
    }

    pub fn set_file_type(mut self, file_type: &TokenFileType) -> Self {
        self.0.file_type = file_type.clone();
        self
    }

    pub fn set_key_stack(mut self, key_stack: Option<Vec<String>>) -> Self {
        self.0.key_stack = key_stack.unwrap_or_default();
        self
    }

    pub fn build(self) -> AutoCompleteToken {
        self.0
    }
}
