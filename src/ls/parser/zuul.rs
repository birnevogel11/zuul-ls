use ropey::Rope;
use tower_lsp::lsp_types::Position;

use crate::ls::parser::word::find_path_word;

use super::word::find_name_word;
use super::WordType;
use yaml_rust2::yaml::Yaml;
use yaml_rust2::yaml::YamlLoader;

const SEARCH_PATTERN: &str = "SeRpAt";

fn retrieve_value_key_stack(value: &Yaml, keys: &mut Vec<String>) -> bool {
    match value {
        Yaml::String(s) => s.contains(SEARCH_PATTERN),
        Yaml::Array(xs) => {
            for x in xs {
                if retrieve_value_key_stack(x, keys) {
                    return true; // TODO: Should we support index operator ? Skip it now
                }
            }
            false
        }
        Yaml::Hash(xs) => {
            for (k, v) in xs {
                let key_name = k.as_str();
                if key_name.is_none() {
                    return false;
                }
                if key_name.unwrap().contains(SEARCH_PATTERN) {
                    return true;
                }
                keys.push(key_name.unwrap().to_string());
                if retrieve_value_key_stack(v, keys) {
                    return true;
                }
            }
            false
        }
        Yaml::Real(_)
        | Yaml::Integer(_)
        | Yaml::Boolean(_)
        | Yaml::Alias(_)
        | Yaml::Null
        | Yaml::BadValue => false,
    }
}

fn insert_search_word(content: &Rope, line: usize, col: usize) -> Rope {
    let cidx = content.line_to_char(line) + col;
    let mut new_content = content.clone();
    new_content.insert(cidx, SEARCH_PATTERN);

    new_content
}

pub fn retrieve_key_stack(content: &Rope, line: usize, col: usize) -> Option<Vec<String>> {
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
                return Some(key_stack);
            }
            key_stack.push(zuul_name.to_string());

            // Traverse all attributes and values of this zuul config
            let zuul_config = raw_zuul_name.get(zuul).unwrap().as_hash()?;
            for (key, value) in zuul_config {
                if key.as_str()?.contains(SEARCH_PATTERN) {
                    return Some(key_stack);
                }

                let mut value_stack = Vec::new();
                if retrieve_value_key_stack(value, &mut value_stack) {
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

    if key_stack.len() >= 2 && key_stack[0] == "job" {
        if key_stack[1] == "name" || key_stack[1] == "parent" {
            let current_word = find_name_word(content, position)?;
            return Some((current_word, vec![WordType::Job]));
        }
        if key_stack[1] == "vars" {
            // TODO: fix it. how to jump lhs value correctly?
            let current_word = find_name_word(content, position)?;
            return Some((current_word, vec![WordType::Variable]));
        }
        if ["run", "pre-run", "post-run", "clean-run"]
            .into_iter()
            .any(|key| key == key_stack[1])
        {
            let current_word = find_path_word(content, position)?;
            return Some((current_word, vec![WordType::Playbook]));
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
