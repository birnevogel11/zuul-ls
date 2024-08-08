use ropey::Rope;
use ropey::RopeSlice;
use tower_lsp::lsp_types::Position;

fn is_char_role(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '/' | '-')
}

fn is_char_var(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.')
}

fn is_char_name(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')
}

fn is_char_path(ch: char) -> bool {
    !matches!(ch, ' ' | '\t' | '\r' | '\n')
}

fn find_token_in_line<T>(line: &RopeSlice, col: usize, is_letter: T) -> Option<String>
where
    T: Fn(char) -> bool,
{
    let mut token_idx: Vec<(usize, usize)> = Vec::new();
    let mut cidx: usize = 0;

    let chars = line.chars().collect::<Vec<_>>();
    while cidx < chars.len() {
        if is_letter(chars[cidx]) {
            let b = cidx;
            while cidx < chars.len() && is_letter(chars[cidx]) {
                cidx += 1;
            }
            let e = cidx;
            token_idx.push((b, e));
        }
        cidx += 1;
    }

    for (b, e) in token_idx {
        if col >= b && col <= e {
            let s = line.slice(b..e).to_string();
            log::info!("token: {:#?}", s);
            return Some(s);
        }
    }
    None
}

fn find_token<T>(content: &Rope, position: &Position, is_letter: T) -> Option<String>
where
    T: Fn(char) -> bool,
{
    find_token_in_line(
        &content.get_line(position.line as usize)?,
        position.character as usize,
        is_letter,
    )
}

pub fn find_role_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_role)
}

pub fn find_var_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_var)
}

pub fn find_name_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_name)
}

pub fn find_path_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_get_current_word_var() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 8);
        let result = find_var_token(&content, &position);

        assert_eq!(result, Some("abc.def".to_string()));
    }

    #[test]
    fn test_get_current_word_role() {
        let content = Rope::from_str("name: subdir/nested-role-name");
        let position = Position::new(0, 8);
        let result = find_role_token(&content, &position);

        assert_eq!(result, Some("subdir/nested-role-name".to_string()));
    }
}
