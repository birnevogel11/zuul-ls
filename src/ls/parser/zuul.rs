use ropey::Rope;
use tower_lsp::lsp_types::Position;

use yaml_rust2::yaml::YamlLoader;

use super::key_stack::{insert_search_word, parse_value, ARRAY_INDEX_KEY, SEARCH_PATTERN};
use super::utils::{find_name_token, find_path_token, find_var_token};
use super::{AutoCompleteToken, TokenFileType, TokenSide, TokenType};

fn retrieve_key_stack(content: &Rope, line: usize, col: usize) -> Option<(Vec<String>, TokenSide)> {
    let search_rope = insert_search_word(content, line, col);
    let content = search_rope.to_string();
    let docs = YamlLoader::load_from_str(&content).ok()?;
    for doc in docs {
        let zuul_config_units = doc.as_vec()?;
        for zuul_config_unit in zuul_config_units {
            // Each zuul config item only contains one key (e.g. job)
            let raw_zuul_name = zuul_config_unit.as_hash()?;
            if raw_zuul_name.keys().len() != 1 {
                return None;
            }

            // Get the key (e.g. job)
            let zuul = *raw_zuul_name.keys().collect::<Vec<_>>().first()?;
            let zuul_name = zuul.as_str()?;
            if zuul_name.contains(SEARCH_PATTERN) {
                return Some((Vec::new(), TokenSide::Left));
            }

            // Get the token
            let value = raw_zuul_name.get(zuul).unwrap();
            let token = parse_value(value, Some(vec![zuul_name.to_string()]));
            if token.is_some() {
                return token;
            }
        }
    }

    None
}

fn parse_project_token(
    content: &Rope,
    position: &Position,
    file_type: TokenFileType,
    token_side: TokenSide,
    mut key_stack: Vec<String>,
) -> Option<AutoCompleteToken> {
    if key_stack.len() < 3 {
        return None;
    }

    if key_stack[2] == "jobs" {
        if (key_stack.len() == 4 && key_stack[3] == ARRAY_INDEX_KEY)
            || (key_stack.len() >= 5 && key_stack[4] == "dependencies")
        {
            return Some(AutoCompleteToken::new(
                find_name_token(content, position)?,
                file_type,
                TokenType::Job,
                token_side,
                key_stack,
            ));
        } else if key_stack.len() >= 6 && key_stack[5] == "vars" {
            let mut var_stack = None;
            if key_stack.len() >= 7 {
                var_stack = Some(key_stack[6..].to_vec());
                key_stack = key_stack[..6].to_vec();
            }

            let token = find_var_token(content, position)?;
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
    }

    None
}

fn parse_job_token(
    content: &Rope,
    position: &Position,
    file_type: TokenFileType,
    token_side: TokenSide,
    mut key_stack: Vec<String>,
) -> Option<AutoCompleteToken> {
    if key_stack.len() < 2 {
        return None;
    }

    match key_stack[1].as_str() {
        "name" | "parent" => {
            let token = find_name_token(content, position)?;
            Some(AutoCompleteToken::new(
                token,
                file_type,
                TokenType::Job,
                token_side,
                key_stack,
            ))
        }
        "vars" => {
            let mut var_stack = None;
            if key_stack.len() >= 3 {
                var_stack = Some(key_stack[2..].to_vec());
                key_stack = key_stack[..2].to_vec();
            }

            let token = find_name_token(content, position)?;
            Some(match token_side {
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
            })
        }
        "run" | "pre-run" | "post-run" => {
            let token = find_path_token(content, position)?;
            Some(AutoCompleteToken::new(
                token,
                file_type,
                TokenType::Playbook,
                token_side,
                key_stack,
            ))
        }
        _ => None,
    }
}

pub fn parse_token_zuul_config(
    file_type: TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let (key_stack, token_side) =
        retrieve_key_stack(content, position.line as usize, position.character as usize)?;
    log::info!(
        "key_stack: {:#?}, token_side: {:#?}",
        &key_stack,
        &token_side
    );

    if key_stack.is_empty() {
        return None;
    }

    if key_stack.len() == 1 && token_side == TokenSide::Left {
        let token = find_name_token(content, position)?;
        return Some(AutoCompleteToken::new(
            token,
            file_type,
            TokenType::ZuulProperty(key_stack[0].clone()),
            token_side,
            key_stack,
        ));
    }

    match key_stack[0].as_str() {
        "job" => parse_job_token(content, position, file_type, token_side, key_stack),
        "project" | "project-template" => {
            parse_project_token(content, position, file_type, token_side, key_stack)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec_str(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_retrieve_property_stack() {
        let content = r#"
- job:
    name: test-job
    parent: parent-job
    "#;
        let xs = retrieve_key_stack(&Rope::from_str(content), 3, 12);
        assert_eq!(xs, Some((to_vec_str(&["job", "parent"]), TokenSide::Right)));
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
        let xs = retrieve_key_stack(&Rope::from_str(content), 6, 14);
        assert_eq!(
            xs,
            Some((to_vec_str(&["job", "vars", "test_var"]), TokenSide::Left))
        );
    }

    #[test]
    fn test_parse_token_zuul_project() {
        let content = r#"
- project:
    check:
      jobs:
        - check_job1
        - check_job2:
            dependencies:
               - dependencies_job_1
            voting: false
    "#;
        let xs = retrieve_key_stack(&Rope::from_str(content), 7, 22);
        assert_eq!(
            xs,
            Some((
                to_vec_str(&[
                    "project",
                    "check",
                    "jobs",
                    "ArRaY_InDeX",
                    "check_job2",
                    "dependencies",
                    "ArRaY_InDeX",
                ]),
                TokenSide::Right
            ))
        )
    }

    #[test]
    fn test_parse_token_zuul_project_var() {
        let content = r#"
- project:
    check:
      jobs:
        - check_job1
        - check_job2:
            vars:
              test_var1: var_value_1
              test_var2:
                nest_var2_1: nest_var2_value
    "#;
        let xs = retrieve_key_stack(&Rope::from_str(content), 9, 22);
        assert!(xs.is_some());

        let (key_stack, token_side) = xs.unwrap();
        let token = parse_project_token(
            &Rope::from_str(content),
            &Position::new(9, 22),
            TokenFileType::ZuulConfig,
            token_side,
            key_stack,
        );
        assert_eq!(
            token,
            Some(AutoCompleteToken {
                value: "nest_var2_1".to_string(),
                file_type: TokenFileType::ZuulConfig,
                token_type: TokenType::VariableWithPrefix(to_vec_str(&["test_var2"])),
                token_side: TokenSide::Left,
                key_stack: to_vec_str(&[
                    "project",
                    "check",
                    "jobs",
                    "ArRaY_InDeX",
                    "check_job2",
                    "vars",
                ]),
            },)
        );
    }

    #[test]
    fn test_parse_token_zuul_project_var2() {
        let content = r#"
- project:
    check:
      jobs:
        - check_job1
        - check_job2:
            vars:
              test_var1: var_value_1
              test_var2:
                nest_var2_1: nest_var2_value
    "#;
        let xs = retrieve_key_stack(&Rope::from_str(content), 7, 22);
        assert!(xs.is_some());

        let (key_stack, token_side) = xs.unwrap();
        let token = parse_project_token(
            &Rope::from_str(content),
            &Position::new(7, 22),
            TokenFileType::ZuulConfig,
            token_side,
            key_stack,
        );
        assert_eq!(
            token,
            Some(AutoCompleteToken {
                value: "test_var1".to_string(),
                file_type: TokenFileType::ZuulConfig,
                token_type: TokenType::VariableWithPrefix(Vec::new()),
                token_side: TokenSide::Left,
                key_stack: to_vec_str(&[
                    "project",
                    "check",
                    "jobs",
                    "ArRaY_InDeX",
                    "check_job2",
                    "vars",
                ]),
            },)
        );
    }
}
