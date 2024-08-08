use ropey::Rope;

use crate::ls::parser::TokenSide;
use yaml_rust2::yaml::Yaml;

pub const SEARCH_PATTERN: &str = "SeRpAt";

pub fn parse_value(value: &Yaml, key_stack: Vec<String>) -> Option<(Vec<String>, TokenSide)> {
    match value {
        Yaml::String(s) => {
            if s.contains(SEARCH_PATTERN) {
                return Some((key_stack.clone(), TokenSide::Right));
            }
        }
        Yaml::Hash(xs) => {
            for (key, value) in xs {
                let key_name = key.as_str()?;
                if key_name.contains(SEARCH_PATTERN) {
                    return Some((key_stack, TokenSide::Left));
                }

                let mut new_key_stack = key_stack.clone();
                new_key_stack.push(key_name.to_string());
                let sub_value = parse_value(value, new_key_stack);
                if sub_value.is_some() {
                    return sub_value;
                }
            }
        }
        Yaml::Array(xs) => {
            for x in xs {
                let sub_value = parse_value(x, key_stack.clone());
                if sub_value.is_some() {
                    return Some((key_stack.clone(), TokenSide::Right));
                }
            }
        }
        _ => {
            return None;
        }
    }

    None
}

pub fn insert_search_word(content: &Rope, line: usize, col: usize) -> Rope {
    let cidx = content.line_to_char(line) + col;
    let mut new_content = content.clone();
    new_content.insert(cidx, SEARCH_PATTERN);

    new_content
}
