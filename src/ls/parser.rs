pub mod ansible;
pub mod word;
pub mod zuul;

use ropey::Rope;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Url;

use crate::ls::file_type::resolve_ls_file_type;
use crate::ls::file_type::LSFileType;

use ansible::parse_word_ansible;
use ansible::parse_word_var;
use zuul::parse_word_zuul_config;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum WordType {
    Variable,
    Role,
    Job,
    ZuulProperty(String),
    Playbook,
}

pub fn parse_word_type(
    uri: &Url,
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<WordType>)> {
    let path = uri.to_file_path().unwrap();
    let file_type = resolve_ls_file_type(&path)?;

    match file_type {
        LSFileType::ZuulConfig => parse_word_zuul_config(content, position),
        LSFileType::Playbooks => parse_word_ansible(content, position),
        LSFileType::AnsibleRoleTasks => parse_word_ansible(content, position),
        LSFileType::AnsibleRoleDefaults => parse_word_var(content, position),
        LSFileType::AnsibleRoleTemplates => parse_word_var(content, position),
    }
}
