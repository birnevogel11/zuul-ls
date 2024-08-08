use tower_lsp::lsp_types::Position;

use super::parser::AutoCompleteToken;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Default)]
pub struct AutoCompleteTokenCache {
    uri_path: String,
    position: Position,
    token: Option<AutoCompleteToken>,
}

impl AutoCompleteTokenCache {
    pub fn new(uri_path: String, position: Position, token: Option<AutoCompleteToken>) -> Self {
        AutoCompleteTokenCache {
            uri_path,
            position,
            token,
        }
    }

    pub fn new_with_append(&self, append_char: char) -> Self {
        let new_position = Position::new(self.position.line, self.position.character + 1);

        AutoCompleteTokenCache {
            position: new_position,
            ..self.clone()
        }
    }

    pub fn is_valid(&self, uri_path: &str, position: &Position) -> bool {
        uri_path == self.uri_path
            && position.line == self.position.line
            && position.character == self.position.character + 1
    }

    pub fn token(&self) -> &Option<AutoCompleteToken> {
        &self.token
    }
}
