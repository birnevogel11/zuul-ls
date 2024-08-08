use std::collections::HashMap;
use std::path::Path;

use hashlink::LinkedHashMap;

use crate::parser::common::StringLoc;
use crate::parser::var_table::{
    merge_var_group, parse_var_group_from_hash, VarGroup, VariableInfo, VariableSource,
};
use crate::parser::yaml::{load_yvalue_from_str, YValue};

pub fn parse_task_vars_internal(
    value: &YValue,
    path: &Path,
    field_name: &str,
    source: &VariableSource,
) -> Option<VarGroup> {
    let tasks = value
        .as_vec()?
        .iter()
        .map_while(|v| v.as_hash())
        .collect::<Vec<_>>();

    let mut var_group = HashMap::new();
    for task in tasks {
        for (key, value) in task {
            let key_name = key.as_str()?;

            let sub_var_group = match key_name {
                "set_fact" | "vars" => {
                    let var_value = value
                        .as_hash()?
                        .into_iter()
                        .map_while(|(key, value)| {
                            if let Some(key_name) = key.as_str() {
                                if key_name != "cachable" {
                                    return Some((key.clone(), value.clone()));
                                }
                            }
                            None
                        })
                        .collect::<LinkedHashMap<_, _>>();
                    parse_var_group_from_hash(&var_value, path, field_name, source).ok()
                }
                "block" | "rescue" | "always" => {
                    parse_task_vars_internal(value, path, field_name, source)
                }
                "register" => {
                    let mut sub_var_group = VarGroup::new();
                    let var_name = value.as_str()?;
                    let var_info = VariableInfo {
                        name: StringLoc::from(value, path),
                        value: "".to_string(),
                        source: source.clone(),
                    };
                    sub_var_group.insert(var_name.to_string(), vec![var_info]);
                    Some(sub_var_group)
                }
                _ => None,
            };

            var_group = merge_var_group(var_group, sub_var_group.unwrap_or_default());
        }
    }

    Some(var_group)
}

pub fn parse_task_vars(
    content: &str,
    path: &Path,
    role_name: &str,
    role_path: &Path,
) -> Option<VarGroup> {
    let docs = load_yvalue_from_str(content).ok()?;
    if docs.len() != 1 {
        return None;
    }
    let source = VariableSource::from_role(role_name, role_path);
    parse_task_vars_internal(&docs[0], path, role_name, &source)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::golden_key_test::TestFiles;

    use super::*;

    #[test]
    fn test_parse_task_vars() {
        let ts = TestFiles::new("ansible_tasks.yaml");
        let xs = parse_task_vars(
            &ts.read_input(),
            &PathBuf::from("/fake/play.yaml"),
            "fake_role",
            &PathBuf::from("/fake/roles/fake_role"),
        );

        let xs = if let Some(xs) = xs {
            let mut var_group = xs.into_iter().collect::<Vec<_>>();
            var_group.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            Some(var_group)
        } else {
            None
        };

        ts.assert_output(&xs);
    }
}
