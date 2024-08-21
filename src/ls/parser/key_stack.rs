use ropey::Rope;
use yaml_rust2::yaml::Yaml;

use crate::ls::parser::TokenSide;
use crate::parser::variable::ARRAY_INDEX_KEY;

pub const SEARCH_PATTERN: &str = "SeRpAt";

fn parse_value_internal(value: &Yaml, key_stack: &mut Vec<String>) -> Option<TokenSide> {
    match value {
        Yaml::String(s) => {
            if s.contains(SEARCH_PATTERN) {
                return Some(TokenSide::Right);
            }
        }
        Yaml::Hash(xs) => {
            for (key, value) in xs {
                let key_name = key.as_str()?;
                if key_name.contains(SEARCH_PATTERN) {
                    return Some(TokenSide::Left);
                }

                key_stack.push(key_name.to_string());
                let sub_value = parse_value_internal(value, key_stack);
                if sub_value.is_some() {
                    return sub_value;
                }
                key_stack.pop();
            }
        }
        Yaml::Array(xs) => {
            for x in xs {
                key_stack.push(ARRAY_INDEX_KEY.to_string());
                let sub_value = parse_value_internal(x, key_stack);
                if sub_value.is_some() {
                    return sub_value;
                }
                key_stack.pop();
            }
        }
        _ => {
            return None;
        }
    }

    None
}

pub fn parse_value(
    value: &Yaml,
    key_stack: Option<Vec<String>>,
) -> Option<(Vec<String>, TokenSide)> {
    let mut key_stack = key_stack.unwrap_or_default();
    let token_side = parse_value_internal(value, &mut key_stack)?;
    Some((key_stack, token_side))
}

pub fn insert_search_word(content: &Rope, line: usize, col: usize) -> Rope {
    let cidx = content.line_to_char(line) + col;
    let mut new_content = content.clone();
    new_content.insert(cidx, SEARCH_PATTERN);

    new_content
}
