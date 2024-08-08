pub mod template;

use ropey::Rope;
use tower_lsp::lsp_types::Position;

use super::word::find_role_word;
use super::word::find_var_word;
use super::WordType;

pub fn parse_word_ansible(content: &Rope, position: &Position) -> Option<(String, Vec<WordType>)> {
    let line_idx = position.line as usize;
    let line = content.get_line(line_idx)?;
    let line_str = line.to_string();

    let search_type = if line_str.contains("role:") {
        WordType::Role
    } else if line_idx >= 1 {
        let prev_line = content.get_line(line_idx - 1).unwrap().to_string();
        if (prev_line.contains("include_role:") || prev_line.contains("import_role:"))
            && line_str.contains("name:")
        {
            WordType::Role
        } else {
            WordType::Variable
        }
    } else {
        WordType::Variable
    };

    let current_word = if search_type == WordType::Role {
        find_role_word(content, position)?
    } else {
        find_var_word(content, position)?
    };

    Some((current_word, vec![search_type]))
}

pub fn parse_word_var(content: &Rope, position: &Position) -> Option<(String, Vec<WordType>)> {
    Some((find_role_word(content, position)?, vec![WordType::Variable]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
    struct TestGetCurrentWordTypeAnsible {
        pub content: Rope,
        pub position: Position,
        pub result: Option<(String, Vec<WordType>)>,
    }

    impl TestGetCurrentWordTypeAnsible {
        fn new(
            content: &str,
            line: u32,
            character: u32,
            current_word: &str,
            search_type: WordType,
        ) -> TestGetCurrentWordTypeAnsible {
            Self::new_result(
                content,
                line,
                character,
                Some((current_word.to_string(), vec![search_type])),
            )
        }

        fn new_result(
            content: &str,
            line: u32,
            character: u32,
            result: Option<(String, Vec<WordType>)>,
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
                WordType::Role,
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
                WordType::Role,
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
                WordType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                r#"
- role: subdir/nested-role-name
            "#,
                1,
                8,
                "subdir/nested-role-name",
                WordType::Role,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                8,
                "abc.def",
                WordType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                1,
                "abc",
                WordType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new_result("abc {{ abc.def }}", 0, 5, None),
        ];

        for input in test_inputs {
            let result = parse_word_ansible(&input.content, &input.position);
            assert_eq!(result, input.result)
        }
    }
}
