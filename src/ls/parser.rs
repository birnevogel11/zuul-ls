pub mod ansible;
pub mod token;
pub mod zuul;

use std::path::Path;

pub use token::{AutoCompleteToken, TokenSide, TokenType};

use ropey::Rope;
use tower_lsp::lsp_types::Position;

use crate::ls::file_type::resolve_ls_file_type;
use crate::ls::file_type::LSFileType;
use ansible::parse_word_ansible;
use ansible::parse_word_var;
use zuul::parse_word_zuul_config;

pub fn parse_token(path: &Path, content: &Rope, position: &Position) -> Option<AutoCompleteToken> {
    let file_type = resolve_ls_file_type(&path)?;

    let ac_token = match file_type {
        LSFileType::ZuulConfig => parse_word_zuul_config(content, position),
        LSFileType::Playbooks => parse_word_ansible(content, position),
        LSFileType::AnsibleRoleTasks => parse_word_ansible(content, position),
        LSFileType::AnsibleRoleDefaults => parse_word_var(content, position),
        LSFileType::AnsibleRoleTemplates => parse_word_var(content, position),
        LSFileType::Unknown => None,
    };

    ac_token.map(|ac_token| AutoCompleteToken {
        file_type,
        ..ac_token
    })
}
