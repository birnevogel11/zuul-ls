use hashlink::LinkedHashMap;
use ropey::Rope;

use crate::ls::parser::TokenSide;
use yaml_rust2::yaml::Yaml;

pub const SEARCH_PATTERN: &str = "SeRpAt";

fn retrieve_value_key_stack(value: &Yaml, keys: &mut Vec<String>) -> (bool, TokenSide) {
    match value {
        Yaml::String(s) => (s.contains(SEARCH_PATTERN), TokenSide::Right),
        Yaml::Array(xs) => {
            for x in xs {
                if retrieve_value_key_stack(x, keys).0 {
                    return (true, TokenSide::Right); // TODO: Should we support index operator ? Skip it now
                }
            }
            (false, TokenSide::default())
        }
        Yaml::Hash(xs) => {
            for (k, v) in xs {
                let key_name = k.as_str();
                if key_name.is_none() {
                    return (false, TokenSide::Left);
                }
                if key_name.unwrap().contains(SEARCH_PATTERN) {
                    return (true, TokenSide::Left);
                }
                keys.push(key_name.unwrap().to_string());
                let check = retrieve_value_key_stack(v, keys);
                if check.0 {
                    return check;
                }
            }
            (false, TokenSide::default())
        }
        Yaml::Real(_)
        | Yaml::Integer(_)
        | Yaml::Boolean(_)
        | Yaml::Alias(_)
        | Yaml::Null
        | Yaml::BadValue => (false, TokenSide::default()),
    }
}

pub fn insert_search_word(content: &Rope, line: usize, col: usize) -> Rope {
    let cidx = content.line_to_char(line) + col;
    let mut new_content = content.clone();
    new_content.insert(cidx, SEARCH_PATTERN);

    new_content
}

pub fn retrieve_config_attribute(
    config: &LinkedHashMap<Yaml, Yaml>,
    key_stack: Vec<String>,
) -> Option<(Vec<String>, Option<Vec<String>>, TokenSide)> {
    let mut key_stack = key_stack;

    for (key, value) in config {
        if key.as_str()?.contains(SEARCH_PATTERN) {
            return Some((key_stack, None, TokenSide::Left));
        }

        let mut value_stack = Vec::new();
        let check = retrieve_value_key_stack(value, &mut value_stack);
        if check.0 {
            key_stack.push(key.as_str()?.to_string());
            let var_stack = if value_stack.is_empty() {
                None
            } else {
                Some(value_stack)
            };
            return Some((key_stack, var_stack, check.1));
        }
    }

    None
}
