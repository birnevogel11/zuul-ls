use ropey::Rope;
use tower_lsp::lsp_types::Position;

use hashlink::LinkedHashMap;
use yaml_rust2::yaml::{Yaml, YamlLoader};

use super::key_stack::{insert_search_word, parse_value, SEARCH_PATTERN};
use super::utils::{find_name_word, find_path_word};
use super::{AutoCompleteToken, TokenFileType, TokenSide, TokenType};

fn retrieve_config_attribute(
    config: &LinkedHashMap<Yaml, Yaml>,
    key_stack: Vec<String>,
) -> Option<(Vec<String>, Option<Vec<String>>, TokenSide)> {
    let mut key_stack = key_stack;

    for (key, value) in config {
        if key.as_str()?.contains(SEARCH_PATTERN) {
            return Some((key_stack, None, TokenSide::Left));
        }

        if let Some((var_stack, token_side)) = parse_value(value, Vec::new()) {
            key_stack.push(key.as_str()?.to_string());
            let var_stack = if var_stack.is_empty() {
                None
            } else {
                Some(var_stack)
            };
            return Some((key_stack, var_stack, token_side));
        }
    }

    None
}

pub fn retrieve_zuul_key_stack(
    content: &Rope,
    line: usize,
    col: usize,
) -> Option<(Vec<String>, Option<Vec<String>>, TokenSide)> {
    let search_rope = insert_search_word(content, line, col);
    let content = search_rope.to_string();
    let docs = YamlLoader::load_from_str(&content).ok()?;
    for doc in docs {
        let zuul_config_units = doc.as_vec()?;
        for zuul_config_unit in zuul_config_units {
            let mut key_stack: Vec<String> = Vec::new();

            // Each zuul config item only contains one key (e.g. job)
            let raw_zuul_name = zuul_config_unit.as_hash()?;
            if raw_zuul_name.keys().len() != 1 {
                return None;
            }

            // Get the key (e.g. job)
            let zuul = *raw_zuul_name.keys().collect::<Vec<_>>().first()?;
            let zuul_name = zuul.as_str()?;
            if zuul_name.contains(SEARCH_PATTERN) {
                return Some((key_stack, None, TokenSide::Left));
            }
            key_stack.push(zuul_name.to_string());

            // Traverse all attributes and values of this zuul config
            let zuul_config = raw_zuul_name.get(zuul).unwrap().as_hash()?;
            let token = retrieve_config_attribute(zuul_config, key_stack);
            if token.is_some() {
                return token;
            }
        }
    }

    None
}

pub fn parse_token_zuul_config(
    file_type: TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let (key_stack, var_stack, token_side) =
        retrieve_zuul_key_stack(content, position.line as usize, position.character as usize)?;
    log::info!(
        "key_stack: {:#?}, token_side: {:#?}",
        &key_stack,
        &token_side
    );

    if key_stack.is_empty() {
        return None;
    }

    if key_stack.len() == 1 && token_side == TokenSide::Left {
        let token = find_name_word(content, position)?;
        return Some(AutoCompleteToken::new(
            token,
            file_type,
            TokenType::ZuulProperty(key_stack[0].clone()),
            token_side,
            key_stack,
        ));
    }

    if key_stack.len() >= 2 && key_stack[0] == "job" {
        if key_stack[1] == "name" || key_stack[1] == "parent" {
            let token = find_name_word(content, position)?;
            return Some(AutoCompleteToken::new(
                token,
                file_type,
                TokenType::Job,
                token_side,
                key_stack,
            ));
        }
        if key_stack[1] == "vars" {
            let token = find_name_word(content, position)?;
            return Some(match token_side {
                TokenSide::Left => AutoCompleteToken::new(
                    token,
                    file_type,
                    TokenType::VariableWithPrefix(var_stack.unwrap_or_default()),
                    token_side,
                    key_stack,
                ),
                TokenSide::Right => AutoCompleteToken::new(
                    token,
                    file_type,
                    TokenType::Variable,
                    token_side,
                    key_stack,
                ),
            });
        }
        if ["run", "pre-run", "post-run"]
            .into_iter()
            .any(|key| key == key_stack[1])
        {
            let token = find_path_word(content, position)?;
            return Some(AutoCompleteToken::new(
                token,
                file_type,
                TokenType::Playbook,
                token_side,
                key_stack,
            ));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve_property_stack() {
        let content = r#"
- job:
    name: test-job
    parent: parent-job
    "#;
        let xs = retrieve_zuul_key_stack(&Rope::from_str(content), 3, 12);
        assert_eq!(
            xs,
            Some((
                (["job", "parent"])
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
                None,
                TokenSide::Right
            ))
        );
    }

    #[test]
    fn test_retrieve_property_stack_left() {
        let content = r#"
- job:
    name: test-job
    parent: parent-job
    vars:
        test_var:
          nested_var: value
    "#;
        let xs = retrieve_zuul_key_stack(&Rope::from_str(content), 6, 14);
        assert_eq!(
            xs,
            Some((
                (["job", "vars"])
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
                Some(
                    ["test_var"]
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                ),
                TokenSide::Left
            ))
        );
    }
}
