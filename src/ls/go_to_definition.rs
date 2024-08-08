use std::path::Path;

use ropey::Rope;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::ls::parser::token::{AutoCompleteToken, TokenType};
use crate::ls::symbols::ZuulSymbol;
use crate::path::{retrieve_repo_path, to_path};

use super::parser::parse_token;

fn get_definition_list_internal(
    symbols: &ZuulSymbol,
    path: &Path,
    ac_token: &AutoCompleteToken,
) -> Option<GotoDefinitionResponse> {
    let token = &ac_token.token;

    match ac_token.token_type {
        TokenType::Variable => {
            if let Some(var_infos) = symbols.vars().get(token) {
                return Some(GotoDefinitionResponse::Array(
                    var_infos
                        .iter()
                        .map(|var| var.name.clone().into())
                        .collect::<Vec<_>>(),
                ));
            }
        }
        TokenType::Job => {
            if let Some(job_locs) = symbols.jobs().get(token) {
                return Some(GotoDefinitionResponse::Array(
                    job_locs
                        .iter()
                        .map(|job_loc| job_loc.clone().into())
                        .collect::<Vec<_>>(),
                ));
            }
        }
        TokenType::Role => {
            if let Some(role) = symbols.role_dirs().get(token) {
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
                let playbook_path = repo_path.join(token);
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
    let ac_word = parse_token(path, content, position)?;
    log::info!("AutoCompleteWord: {:#?}", &ac_word);
    get_definition_list_internal(symbols, path, &ac_word)
}
