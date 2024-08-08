use ropey::Rope;
use tower_lsp::lsp_types::Position;

use super::utils::{find_role_word, find_var_word};
use super::{AutoCompleteToken, TokenFileType, TokenType};

pub fn parse_token_ansible(
    file_type: TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let line_idx = position.line as usize;
    let line = content.get_line(line_idx)?;
    let line_str = line.to_string();

    let token_type = if line_str.contains("role:") {
        TokenType::Role
    } else if line_idx >= 1 {
        let prev_line = content.get_line(line_idx - 1).unwrap().to_string();
        if (prev_line.contains("include_role:") || prev_line.contains("import_role:"))
            && line_str.contains("name:")
        {
            TokenType::Role
        } else {
            TokenType::Variable
        }
    } else {
        TokenType::Variable
    };

    let token = if token_type == TokenType::Role {
        find_role_word(content, position)?
    } else {
        find_var_word(content, position)?
    };

    Some(AutoCompleteToken::new_simple(token, file_type, token_type))
}

pub fn parse_token_var(
    file_type: TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    Some(AutoCompleteToken::new_simple(
        find_var_word(content, position)?,
        file_type,
        TokenType::Variable,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
    struct TestGetCurrentWordTypeAnsible {
        pub content: Rope,
        pub position: Position,
        pub result: Option<AutoCompleteToken>,
    }

    impl TestGetCurrentWordTypeAnsible {
        fn new(
            content: &str,
            line: u32,
            character: u32,
            current_word: &str,
            search_type: TokenType,
        ) -> TestGetCurrentWordTypeAnsible {
            Self::new_result(
                content,
                line,
                character,
                Some(AutoCompleteToken::new_simple(
                    current_word.to_string(),
                    TokenFileType::AnsibleRoleTasks {
                        defaults_path: None,
                    },
                    search_type,
                )),
            )
        }

        fn new_result(
            content: &str,
            line: u32,
            character: u32,
            result: Option<AutoCompleteToken>,
        ) -> TestGetCurrentWordTypeAnsible {
            TestGetCurrentWordTypeAnsible {
                content: content.to_string().into(),
                position: Position::new(line, character),
                result,
            }
        }
    }

    #[test]
    fn test_get_current_word_type_ansible() {
        let test_inputs = [
            TestGetCurrentWordTypeAnsible::new(
                r#"
- name: call one role
  include_role:
    name: subdir/nested-role-name
            "#,
                3,
                15,
                "subdir/nested-role-name",
                TokenType::Role,
            ),
            TestGetCurrentWordTypeAnsible::new(
                r#"
- name: call one role
  import_role:
    name: subdir/nested-role-name
             "#,
                3,
                15,
                "subdir/nested-role-name",
                TokenType::Role,
            ),
            TestGetCurrentWordTypeAnsible::new(
                r#"
- name: call one role
  set_fact:
    name: subdir/nested-role-name
             "#,
                3,
                15,
                "subdir",
                TokenType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                r#"
- role: subdir/nested-role-name
            "#,
                1,
                8,
                "subdir/nested-role-name",
                TokenType::Role,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                8,
                "abc.def",
                TokenType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                1,
                "abc",
                TokenType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new_result("abc {{ abc.def }}", 0, 5, None),
        ];

        for input in test_inputs {
            let result = parse_token_ansible(
                TokenFileType::AnsibleRoleTasks {
                    defaults_path: None,
                },
                &input.content,
                &input.position,
            );
            assert_eq!(result, input.result)
        }
    }
}
