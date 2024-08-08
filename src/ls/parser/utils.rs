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

fn find_token_in_line<T>(
    line: &RopeSlice,
    col: usize,
    is_char: T,
) -> Option<(String, (usize, usize))>
where
    T: Fn(char) -> bool,
{
    let mut token_idx: Vec<(usize, usize)> = Vec::new();
    let mut cidx: usize = 0;

    let chars = line.chars().collect::<Vec<_>>();
    while cidx < chars.len() {
        if is_char(chars[cidx]) {
            let b = cidx;
            while cidx < chars.len() && is_char(chars[cidx]) {
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
            return Some((s, (b, e)));
        }
    }
    None
}

fn find_token<T>(content: &Rope, position: &Position, is_char: T) -> Option<String>
where
    T: Fn(char) -> bool,
{
    find_token_in_line(
        &content.get_line(position.line as usize)?,
        position.character as usize,
        is_char,
    )
    .map(|(token, _)| token)
}

pub fn find_role_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_role)
}

pub fn find_name_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_name)
}

pub fn find_path_token(content: &Rope, position: &Position) -> Option<String> {
    find_token(content, position, is_char_path)
}

pub fn find_var_token(content: &Rope, position: &Position) -> Option<String> {
    let (raw_token, (bidx, _)) = find_token_in_line(
        &content.get_line(position.line as usize)?,
        position.character as usize,
        is_char_var,
    )?;
    if !raw_token.contains('.') {
        return Some(raw_token);
    }
    let offset = position.character as usize - bidx;

    let mut current_len = 0;
    let mut xs_end_slice_idx = 0;
    let xs: Vec<_> = raw_token.split('.').collect();
    for (idx, x) in xs.iter().enumerate() {
        current_len += x.len() + 1;
        if offset < current_len {
            xs_end_slice_idx = idx + 1;
            break;
        }
    }

    Some(xs[..xs_end_slice_idx].join("."))
}

pub fn find_var_token2(content: &Rope, position: &Position) -> Option<Vec<String>> {
    let (raw_token, (bidx, _)) = find_token_in_line(
        &content.get_line(position.line as usize)?,
        position.character as usize,
        is_char_var,
    )?;
    if !raw_token.contains('.') {
        return Some(vec![raw_token]);
    }
    let offset = position.character as usize - bidx;

    let mut current_len = 0;
    let mut xs_end_slice_idx = 0;
    let xs: Vec<_> = raw_token.split('.').collect();
    for (idx, x) in xs.iter().enumerate() {
        current_len += x.len() + 1;
        if offset < current_len {
            xs_end_slice_idx = idx + 1;
            break;
        }
    }

    Some(
        xs[..xs_end_slice_idx]
            .iter()
            .map(|x| x.to_string())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_get_var_token() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 8);
        let result = find_var_token(&content, &position);

        assert_eq!(result, Some("abc".to_string()));
    }

    #[test]
    fn test_get_var_token_with_dot() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 12);
        let result = find_var_token(&content, &position);

        assert_eq!(result, Some("abc.def".to_string()));
    }

    #[test]
    fn test_get_role_token() {
        let content = Rope::from_str("name: subdir/nested-role-name");
        let position = Position::new(0, 8);
        let result = find_role_token(&content, &position);

        assert_eq!(result, Some("subdir/nested-role-name".to_string()));
    }
}
