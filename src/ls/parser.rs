use ropey::Rope;
use ropey::RopeSlice;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Url;

use crate::ls::path::get_file_type;
use crate::ls::path::LSFileType;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum SearchType {
    Variable,
    Role,
    Job,
}

fn is_letter_role(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '/' | '-')
}

fn is_letter_var(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.')
}

fn find_current_word<T>(line: &RopeSlice, col: usize, is_letter: T) -> Option<String>
where
    T: Fn(char) -> bool,
{
    let mut token_index = Vec::new();
    let mut b = -1;
    let mut e = -1;
    for (cidx, c) in line.chars().enumerate() {
        if !is_letter(c) {
            if b != -1 {
                e = cidx as i32;
                let ub = b as usize;
                let ue = e as usize;
                log::info!("b: {}, e: {}, token: {}", b, e, line.slice(ub..ue));
                token_index.push((ub, ue));

                b = -1;
                e = -1;
            }
        } else if is_letter(c) && b == -1 {
            b = cidx as i32;
        }
    }
    if b != -1 {
        let ub = b as usize;
        let ue = line.chars().len();
        log::info!("b: {}, e: {}, token: {}", b, e, line.slice(ub..ue));
        token_index.push((ub, ue));
    }
    for (b, e) in token_index {
        if col >= b && col <= e {
            let s = line.slice(b..e).to_string();
            log::info!("token: {:#?}", s);
            return Some(s);
        }
    }
    None
}

pub fn get_current_word_role(line: &RopeSlice, col: usize) -> Option<String> {
    find_current_word(line, col, is_letter_role)
}

pub fn get_current_word_var(line: &RopeSlice, col: usize) -> Option<String> {
    find_current_word(line, col, is_letter_var)
}

pub fn get_current_word_type_ansible(
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<SearchType>)> {
    let line_idx = position.line as usize;
    let line = content.get_line(line_idx)?;
    let line_str = line.to_string();

    let search_type = if line_str.contains("role:") {
        SearchType::Role
    } else if line_idx >= 1 {
        let prev_line = content.get_line(line_idx - 1).unwrap().to_string();
        if (prev_line.contains("include_role:") || prev_line.contains("import_role:"))
            && line_str.contains("name:")
        {
            SearchType::Role
        } else {
            SearchType::Variable
        }
    } else {
        SearchType::Variable
    };

    let current_word = if search_type == SearchType::Role {
        get_current_word_role(&line, position.character as usize)?
    } else {
        get_current_word_var(&line, position.character as usize)?
    };

    Some((current_word, vec![search_type]))
}

pub fn get_current_word_type_ansible_var(
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<SearchType>)> {
    Some((
        get_current_word_var(
            &content.get_line(position.line as usize)?,
            position.character as usize,
        )?,
        vec![SearchType::Variable],
    ))
}

pub fn parse_autocomplete_type(
    file_type: LSFileType,
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<SearchType>)> {
    match file_type {
        LSFileType::ZuulConfig => None, // TODO: implement it
        LSFileType::Playbooks => get_current_word_type_ansible(content, position),
        LSFileType::AnsibleRoleTasks => get_current_word_type_ansible(content, position),
        LSFileType::AnsibleRoleDefaults => get_current_word_type_ansible_var(content, position),
        LSFileType::AnsibleRoleTemplates => get_current_word_type_ansible_var(content, position),
    }
}

pub fn parse_current_word_type(
    uri: &Url,
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<SearchType>)> {
    let path = uri.to_file_path().unwrap();
    let file_type = get_file_type(&path)?;
    parse_autocomplete_type(file_type, content, position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_word_var() {
        let content = Rope::from_str("abc {{ abc.def }}");
        let position = Position::new(0, 8);
        let result = get_current_word_var(
            &content.get_line(position.line as usize).unwrap(),
            position.character as usize,
        );

        assert_eq!(result, Some("abc.def".to_string()));
    }

    #[test]
    fn test_get_current_word_role() {
        let content = Rope::from_str("name: subdir/nested-role-name");
        let position = Position::new(0, 8);
        let result = get_current_word_role(
            &content.get_line(position.line as usize).unwrap(),
            position.character as usize,
        );

        assert_eq!(result, Some("subdir/nested-role-name".to_string()));
    }

    #[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
    struct TestGetCurrentWordTypeAnsible {
        pub content: Rope,
        pub position: Position,
        pub result: Option<(String, Vec<SearchType>)>,
    }

    impl TestGetCurrentWordTypeAnsible {
        fn new(
            content: &str,
            line: u32,
            character: u32,
            current_word: &str,
            search_type: SearchType,
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
            result: Option<(String, Vec<SearchType>)>,
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
                SearchType::Role,
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
                SearchType::Role,
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
                SearchType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                r#"
- role: subdir/nested-role-name
            "#,
                1,
                8,
                "subdir/nested-role-name",
                SearchType::Role,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                8,
                "abc.def",
                SearchType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new(
                "abc {{ abc.def }}",
                0,
                1,
                "abc",
                SearchType::Variable,
            ),
            TestGetCurrentWordTypeAnsible::new_result("abc {{ abc.def }}", 0, 5, None),
        ];

        for input in test_inputs {
            let result = get_current_word_type_ansible(&input.content, &input.position);
            assert_eq!(result, input.result)
        }
    }
}

// fn is_item_begin(line: &str) -> Option<usize> {
//     if line.trim().starts_with("- ") {
//         let indent_size = line.chars().take_while(|c| *c == ' ').count();
//         Some(indent_size)
//     } else {
//         None
//     }
// }
//
// fn find_begin(content: &Rope, line_idx: usize) -> Option<(usize, usize)> {
//     let mut lidx = line_idx;
//     while lidx > 0 {
//         if let Some(indent_size) = is_item_begin(&content.line(lidx).to_string()) {
//             return Some((lidx, indent_size));
//         }
//         lidx -= 1;
//     }
//     is_item_begin(&content.line(0).to_string()).map(|indent_size| (0, indent_size))
// }
//
// fn find_end(content: &Rope, line_idx: usize, indent_size: usize) -> Option<usize> {
//     let mut pattern = " ".repeat(indent_size);
//     pattern.push_str("- ");
//
//     let mut lidx = line_idx;
//     while lidx < content.lines().len() {
//         if content.line(lidx).to_string().trim().starts_with(&pattern) {
//             return Some(lidx);
//         }
//         lidx += 1;
//     }
//     Some(content.lines().len() - 1)
// }
//
// fn find_begin_end(content: &Rope, line_idx: usize) -> Option<(usize, usize)> {
//     let (begin_idx, indent_size) = find_begin(content, line_idx)?;
//     let end_idx = find_end(content, line_idx, indent_size)?;
//     Some((begin_idx, end_idx))
// }
//
// pub fn parse_ansible_block(content: &Rope, position: &Position) {
//     log::info!("content: {:#?}", content);
//     log::info!("position: {:#?}", position);
//
//     let lidx = position.line as usize;
// }
