use std::collections::HashMap;
use std::path::Path;

use crate::parser::common::StringLoc;
use crate::parser::var_table::{
    collect_variables, group_variables, parse_var_table, VarGroup, VariableInfo, VariableSource,
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
            match key_name {
                "set_fact" | "vars" => {
                    let vt = parse_var_table(value, path, field_name).ok()?;
                    let sub_var_info = collect_variables("", &vt, source)
                        .into_iter()
                        .filter(|(name, _)| name != "cachable")
                        .collect::<HashMap<_, _>>();
                    var_group = group_variables(var_group, sub_var_info);
                    // var_info.extend(sub_var_info);
                }
                "block" | "rescue" | "always" => {
                    let sub_var_group = parse_task_vars_internal(value, path, field_name, source)?;
                    sub_var_group.into_iter().for_each(|(name, var_info)| {
                        match var_group.get_mut(&name) {
                            Some(info) => {
                                info.extend(var_info);
                            }
                            None => {
                                var_group.insert(name, var_info);
                            }
                        }
                    });
                }
                "register" => {
                    let var_name = value.as_str()?;
                    let var_info = VariableInfo {
                        name: StringLoc::from(value, path),
                        value: "".to_string(),
                        source: source.clone(),
                    };
                    match var_group.get_mut(var_name) {
                        Some(info) => {
                            info.push(var_info);
                        }
                        _ => {
                            var_group.insert(var_name.to_string(), vec![var_info]);
                        }
                    };
                }
                _ => {}
            };
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
