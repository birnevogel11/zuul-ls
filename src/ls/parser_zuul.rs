use ropey::Rope;

use crate::parser::yaml::load_yvalue_from_str;
use crate::parser::yaml::YValueYaml;

pub fn parse_zuul_config(content: &Rope, line: usize, col: usize) -> Option<Vec<String>> {
    let content = content.to_string();
    let roots = load_yvalue_from_str(&content).ok()?;
    println!("roots: {:#?}", roots);
    let mut key_stack: Vec<String> = Vec::new();
    for root in roots {
        let xs = root.as_vec()?;
        for x in xs {
            let x = x.as_hash()?;
            if x.keys().len() != 1 {
                return None;
            }
            let zuul_key = *x.keys().collect::<Vec<_>>().first().unwrap();
            if zuul_key.line() == line {
                return Some(key_stack);
            }
            key_stack.push(zuul_key.as_str()?.to_string());

            let zuul_value = x.get(zuul_key).unwrap().as_hash()?;
            let mut prev_key_name = "".to_string();
            for (key, _) in zuul_value {
                let key_name = key.as_str()?;
                if key.line() == line && col < key.col() + key_name.len() && col >= key.col() {
                    return Some(key_stack);
                }
                if key.line() == line {
                    key_stack.push(key_name.to_string());
                    return Some(key_stack);
                }
                if key.line() > line {
                    key_stack.push(prev_key_name);
                    return Some(key_stack);
                }

                prev_key_name = key_name.to_string();
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zuul_config() {
        let content = r#"
- job:
    name: test-job
    parent: parent-job
"#;
        let xs = parse_zuul_config(&Rope::from_str(content), 4, 16);
        println!("xs: {:#?}", xs);
    }
}
