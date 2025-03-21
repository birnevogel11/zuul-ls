use ropey::Rope;
use ropey::RopeSlice;
use tower_lsp::lsp_types::Position;

fn is_char_var(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.')
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

pub fn find_var_token(content: &Rope, position: &Position) -> Option<Vec<String>> {
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

    fn to_vec_str(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_get_var_token() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 8);
        let result = find_var_token(&content, &position);

        assert_eq!(result, Some(to_vec_str(&["abc"])));
    }

    #[test]
    fn test_get_var_token_with_dot() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 12);
        let result = find_var_token(&content, &position);

        assert_eq!(result, Some(to_vec_str(&["abc", "def"])));
    }
}
