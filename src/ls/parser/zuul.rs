use ropey::Rope;
use tower_lsp::lsp_types::Position;

use super::word::find_name_word;
use super::WordType;
use crate::parser::yaml::load_yvalue_from_str;
use crate::parser::yaml::YValue;
use crate::parser::yaml::YValueYaml;

fn is_in_loc(value: &YValue, line: usize, col: usize) -> bool {
    match value.as_str() {
        Some(s) => value.line() == line && col >= value.col() && col < value.col() + s.len(),
        None => false,
    }
}

fn retrieve_value_key_stack(
    value: &YValue,
    line: usize,
    col: usize,
    keys: &mut Vec<String>,
) -> bool {
    match value.value() {
        YValueYaml::String(_) => is_in_loc(value, line, col),
        YValueYaml::Array(xs) => {
            for x in xs {
                if retrieve_value_key_stack(x, line, col, keys) {
                    return true; // TODO: fix it
                }
            }
            false
        }
        YValueYaml::Hash(xs) => {
            for (k, v) in xs {
                let key_name = k.as_str();
                if key_name.is_none() {
                    return false;
                }
                if is_in_loc(k, line, col) {
                    return true;
                }
                keys.push(key_name.unwrap().to_string());
                if retrieve_value_key_stack(v, line, col, keys) {
                    return true;
                }
            }
            false
        }
        YValueYaml::Real(_)
        | YValueYaml::Integer(_)
        | YValueYaml::Boolean(_)
        | YValueYaml::Alias(_)
        | YValueYaml::Null
        | YValueYaml::BadValue => false,
    }
}

// FIXME: try to handle multiline string
pub fn retrieve_key_stack(content: &Rope, line: usize, col: usize) -> Option<Vec<String>> {
    let content = content.to_string();
    let docs = load_yvalue_from_str(&content).ok()?;
    for doc in docs {
        let zuul_config_units = doc.as_vec()?;
        for zuul_config_unit in zuul_config_units {
            let mut key_stack: Vec<String> = Vec::new();

            let raw_zuul_name = zuul_config_unit.as_hash()?;
            if raw_zuul_name.keys().len() != 1 {
                return None;
            }
            let zuul_name = *raw_zuul_name.keys().collect::<Vec<_>>().first().unwrap();
            if zuul_name.line() == line {
                return Some(key_stack);
            }
            key_stack.push(zuul_name.as_str()?.to_string());

            let zuul_config = raw_zuul_name.get(zuul_name).unwrap().as_hash()?;
            for (key, value) in zuul_config {
                if is_in_loc(key, line, col) {
                    return Some(key_stack);
                }

                let mut value_stack = Vec::new();
                if retrieve_value_key_stack(value, line, col, &mut value_stack) {
                    key_stack.push(key.as_str()?.to_string());
                    key_stack.extend(value_stack);
                    return Some(key_stack);
                }
            }
        }
    }

    None
}

pub fn parse_word_zuul_config(
    content: &Rope,
    position: &Position,
) -> Option<(String, Vec<WordType>)> {
    let key_stack =
        retrieve_key_stack(content, position.line as usize, position.character as usize)?;
    log::info!("key_stack: {:#?}", key_stack);

    if key_stack.len() <= 1 {
        return None;
    }

    if key_stack.len() >= 2 {
        let current_word = find_name_word(content, position)?;
        if key_stack[0] == "job" {
            if key_stack[1] == "name" || key_stack[1] == "parent" {
                return Some((current_word, vec![WordType::Job]));
            }
            if key_stack[1] == "vars" {
                return Some((current_word, vec![WordType::Variable]));
            }
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
        let xs = retrieve_key_stack(&Rope::from_str(content), 3, 12);
        assert_eq!(
            xs,
            Some(
                (["job", "parent"])
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            )
        );
        // println!("xs: {:#?}", xs);
    }
}
