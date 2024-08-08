use ropey::Rope;
use tower_lsp::lsp_types::Position;
use yaml_rust2::Yaml;

use super::key_stack::insert_search_word;
use super::key_stack::parse_value;
use super::key_stack::SEARCH_PATTERN;
use super::utils::{find_role_word, find_var_word};
use super::TokenSide;
use super::{AutoCompleteToken, TokenFileType, TokenType};
use yaml_rust2::yaml::YamlLoader;

fn parse_var(
    value: &Yaml,
    file_type: &TokenFileType,
    content: &Rope,
    position: &Position,
    key_stack: Option<Vec<String>>,
) -> Option<AutoCompleteToken> {
    if let Some((var_stack, token_side)) = parse_value(value, Vec::new()) {
        let token_type = match token_side {
            TokenSide::Left => TokenType::VariableWithPrefix(var_stack),
            TokenSide::Right => TokenType::Variable,
        };
        return Some(AutoCompleteToken::new(
            find_var_word(content, position)?,
            file_type.clone(),
            token_type,
            token_side,
            key_stack.unwrap_or_default(),
        ));
    }

    None
}

fn parse_ansible_tasks(
    doc: &Yaml,
    file_type: &TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let tasks = doc.as_vec()?;
    for raw_task in tasks {
        let mut key_stack: Vec<String> = Vec::new();
        let task = raw_task.as_hash()?;
        for (key, value) in task {
            // Check key
            let key_name = key.as_str()?;
            if key_name.contains(SEARCH_PATTERN) {
                return None;
            }

            // Check nested tasks first
            if ["block", "rescue", "always"].contains(&key_name) {
                if let Some(token) = parse_ansible_tasks(value, file_type, content, position) {
                    key_stack.push(key_name.to_string());
                    key_stack.extend(token.key_stack.clone());
                    return Some(AutoCompleteToken { key_stack, ..token });
                }
            }

            // Check value
            if let Some((value_stack, token_side)) = parse_value(value, Vec::new()) {
                key_stack.push(key_name.to_string());

                if [
                    "include_role",
                    "import_role",
                    "ansible.builtin.include_role",
                    "ansible.builtin.import_role",
                ]
                .contains(&key_name)
                {
                    if value_stack.len() == 1
                        && value_stack[0] == "name"
                        && token_side == TokenSide::Right
                    {
                        key_stack.extend(value_stack);

                        return Some(AutoCompleteToken::new(
                            find_role_word(content, position)?,
                            file_type.clone(),
                            TokenType::Role,
                            token_side,
                            key_stack,
                        ));
                    }
                } else if ["set_fact", "ansible.builtin.set_fact", "vars"].contains(&key_name) {
                    let token_type = match token_side {
                        TokenSide::Left => TokenType::VariableWithPrefix(value_stack),
                        TokenSide::Right => TokenType::Variable,
                    };
                    return Some(AutoCompleteToken::new(
                        find_var_word(content, position)?,
                        file_type.clone(),
                        token_type,
                        token_side,
                        key_stack,
                    ));
                } else {
                    return Some(AutoCompleteToken::new(
                        find_var_word(content, position)?,
                        file_type.clone(),
                        TokenType::Variable,
                        token_side,
                        key_stack,
                    ));
                }
            }
        }
    }

    None
}

fn parse_roles(
    doc: &Yaml,
    file_type: &TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let roles = doc.as_vec()?;
    for raw_role_task in roles {
        let role_task = raw_role_task.as_hash()?;
        for (key, value) in role_task {
            let key_name = key.as_str()?;
            if key_name.contains(SEARCH_PATTERN) {
                return Some(AutoCompleteToken::new(
                    find_var_word(content, position)?,
                    file_type.clone(),
                    TokenType::Variable,
                    TokenSide::Left,
                    Vec::new(),
                ));
            }

            if let Some((var_stack, token_side)) = parse_value(value, Vec::new()) {
                if key_name == "role" && token_side == TokenSide::Right {
                    return Some(AutoCompleteToken::new(
                        find_role_word(content, position)?,
                        file_type.clone(),
                        TokenType::Role,
                        TokenSide::Right,
                        Vec::new(),
                    ));
                } else {
                    let token_type = match token_side {
                        TokenSide::Left => TokenType::VariableWithPrefix(var_stack),
                        TokenSide::Right => TokenType::Variable,
                    };
                    return Some(AutoCompleteToken::new(
                        find_var_word(content, position)?,
                        file_type.clone(),
                        token_type,
                        token_side,
                        Vec::new(),
                    ));
                }
            }
        }
    }

    None
}

fn parse_playbook(
    doc: &Yaml,
    file_type: &TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    if let Some(xs) = doc.as_vec() {
        for x in xs {
            if let Some(ys) = x.as_hash() {
                for (key, value) in ys {
                    let key_name = key.as_str()?;
                    if key_name.contains(SEARCH_PATTERN) {
                        return None;
                    }

                    let result = match key_name {
                        "tasks" | "pre_tasks" | "post_tasks" => {
                            parse_ansible_tasks(value, file_type, content, position)
                        }
                        "roles" => parse_roles(value, file_type, content, position),
                        "vars" => parse_var(value, file_type, content, position, None),
                        _ => None,
                    };

                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
    }

    None
}

pub fn parse_token_ansible(
    file_type: TokenFileType,
    content: &Rope,
    position: &Position,
) -> Option<AutoCompleteToken> {
    let search_rope =
        insert_search_word(content, position.line as usize, position.character as usize);
    let docs = YamlLoader::load_from_str(&search_rope.to_string()).ok()?;
    docs.iter().find_map(|doc| match &file_type {
        TokenFileType::AnsibleRoleDefaults => parse_var(doc, &file_type, content, position, None),
        TokenFileType::AnsibleRoleTemplates { .. } => Some(AutoCompleteToken::new_simple(
            find_var_word(content, position)?,
            file_type.clone(),
            TokenType::Variable,
        )),
        TokenFileType::AnsibleRoleTasks { .. } => {
            parse_ansible_tasks(doc, &file_type, content, position)
        }
        TokenFileType::Playbooks => parse_playbook(doc, &file_type, content, position),
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN_FILE_TYPE_PLAYBOOKS: TokenFileType = TokenFileType::Playbooks;
    const TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS: TokenFileType = TokenFileType::AnsibleRoleTasks {
        defaults_path: None,
    };
    const TOKEN_FILE_TYPE_ANSIBLE_ROLE_DEFAULTS: TokenFileType = TokenFileType::AnsibleRoleDefaults;
    const TOKEN_FILE_TYPE_ANSIBLE_ROLE_TEMPLATES: TokenFileType =
        TokenFileType::AnsibleRoleTemplates {
            tasks_path: None,
            defaults_path: None,
        };

    #[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
    struct TestParseTokenAnsible {
        pub content: Rope,
        pub position: Position,
        pub result: Option<AutoCompleteToken>,

        create_token: bool,
        value: String,
        key_stack: Vec<String>,
        file_type: TokenFileType,
        token_side: TokenSide,
        token_type: TokenType,
        var_stack: Vec<String>,
    }

    impl TestParseTokenAnsible {
        fn set_content(mut self, content: &str) -> Self {
            self.content = content.into();
            self
        }

        fn set_location(mut self, line: u32, character: u32) -> Self {
            self.position = Position::new(line, character);
            self
        }

        fn set_value(mut self, value: &str) -> Self {
            self.value = value.to_string();
            self
        }

        fn set_file_type(mut self, file_type: &TokenFileType) -> Self {
            self.file_type = file_type.clone();
            self
        }

        fn set_token_side(mut self, token_side: TokenSide) -> Self {
            self.token_side = token_side;
            self
        }

        fn set_token_type(mut self, token_type: TokenType) -> Self {
            self.token_type = match token_type {
                TokenType::Variable => {
                    if self.var_stack.is_empty() {
                        token_type
                    } else {
                        TokenType::VariableWithPrefix(self.var_stack.clone())
                    }
                }
                _ => token_type,
            };

            self
        }

        fn append_key_stack(mut self, key: &str) -> Self {
            self.key_stack.push(key.to_string());
            self
        }

        fn append_var_stack(mut self, var: &str) -> Self {
            self.var_stack.push(var.to_string());
            self
        }

        fn create_token(mut self) -> Self {
            self.result = Some(AutoCompleteToken::new(
                self.value.clone(),
                self.file_type.clone(),
                self.token_type.clone(),
                self.token_side,
                self.key_stack.clone(),
            ));
            self
        }

        fn build(self) -> TestParseTokenAnsible {
            self
        }

        fn test(self) {
            let result = parse_token_ansible(self.file_type.clone(), &self.content, &self.position);
            assert_eq!(result, self.result)
        }
    }

    #[test]
    fn test_get_role_with_task() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: call one role
  include_role:
    name: subdir/nested-role-name
             "#,
            )
            .set_location(3, 15)
            .set_value("subdir/nested-role-name")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Role)
            .append_key_stack("include_role")
            .append_key_stack("name")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_role_not_found() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: call one role
  include_role:
    name: subdir/nested-role-name
             "#,
            )
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_location(3, 9)
            .build()
            .test();
    }

    #[test]
    fn test_variable_with_task() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  set_fact:
    name: subdir
             "#,
            )
            .set_location(3, 15)
            .set_value("subdir")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Variable)
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_with_task_left() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  set_fact:
    name: subdir
             "#,
            )
            .set_location(3, 7)
            .set_value("name")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::VariableWithPrefix(Vec::new()))
            .set_token_side(TokenSide::Left)
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_with_task_left_nested() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  set_fact:
    name:
      nested: subdir
             "#,
            )
            .set_location(4, 9)
            .set_value("nested")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .append_var_stack("name")
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Left)
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_with_task_right_nested() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  set_fact:
    name:
      nested: subdir
             "#,
            )
            .set_location(4, 18)
            .set_value("subdir")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Right)
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_with_task2() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  set_fact:
    name: "{{ abc.def }}_123"
             "#,
            )
            .set_location(3, 20)
            .set_value("abc.def")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Variable)
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_playbook_with_roles() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- hosts: all
  roles:
    - role: subdir/nested-role
             "#,
            )
            .set_location(3, 15)
            .set_value("subdir/nested-role")
            .set_file_type(&TOKEN_FILE_TYPE_PLAYBOOKS)
            .set_token_type(TokenType::Role)
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_block() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  block:
    - name: nested set_fact
      set_fact:
        name:
          nested: subdir
             "#,
            )
            .set_location(6, 22)
            .set_value("subdir")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Right)
            .append_key_stack("block")
            .append_key_stack("set_fact")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_block_var() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
- name: set a new variable
  block:
    - name: nested set_fact
      set_fact:
        name:
          nested: subdir
      vars:
        var_name:
          nested_var: nested_var_value
             "#,
            )
            .set_location(9, 34)
            .set_value("nested_var_value")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TASKS)
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Right)
            .append_key_stack("block")
            .append_key_stack("vars")
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_defaults() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
var_name_1: value1
var_name_2: value2
var_name_3: value3
             "#,
            )
            .set_location(2, 17)
            .set_value("value2")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_DEFAULTS)
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Right)
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_defaults_left() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
var_name_1: value1
var_name_2: value2
var_name_3: value3
             "#,
            )
            .set_location(2, 8)
            .set_value("var_name_2")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_DEFAULTS)
            .set_token_type(TokenType::VariableWithPrefix(Vec::new()))
            .set_token_side(TokenSide::Left)
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_defaults_left_nested() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
var_name_1: value1
var_name_2:
    nested_var_name2: value2
var_name_3: value3
             "#,
            )
            .set_location(3, 8)
            .set_value("nested_var_name2")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_DEFAULTS)
            .append_var_stack("var_name_2")
            .set_token_type(TokenType::Variable)
            .set_token_side(TokenSide::Left)
            .create_token()
            .build()
            .test();
    }

    #[test]
    fn test_variable_in_template() {
        TestParseTokenAnsible::default()
            .set_content(
                r#"
abc def {{ ghi }}
             "#,
            )
            .set_location(1, 6)
            .set_value("def")
            .set_file_type(&TOKEN_FILE_TYPE_ANSIBLE_ROLE_TEMPLATES)
            .set_token_type(TokenType::Variable)
            .create_token()
            .build()
            .test();
    }
}
