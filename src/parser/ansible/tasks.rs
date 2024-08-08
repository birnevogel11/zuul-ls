use std::path::Path;

use hashlink::LinkedHashMap;

use crate::parser::common::StringLoc;
use crate::parser::variable::{VariableGroup, VariableGroupInfo, VariableTable};
use crate::parser::yaml::{load_yvalue_from_str, YValue};

pub fn parse_task_vars_internal(
    value: &YValue,
    path: &Path,
    field_name: &str,
    source: &crate::parser::variable::VariableSource,
) -> Option<VariableGroup> {
    let tasks = value
        .as_vec()?
        .iter()
        .map_while(|v| v.as_hash())
        .collect::<Vec<_>>();

    let mut var_group = VariableGroup::default();
    for task in tasks {
        for (key, value) in task {
            let key_name = key.as_str()?;

            let sub_var_group: Option<VariableGroup> = match key_name {
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
                    Some(
                        VariableTable::parse_map(&var_value, path, field_name, source)
                            .ok()?
                            .into(),
                    )
                }
                "block" | "rescue" | "always" => {
                    parse_task_vars_internal(value, path, field_name, source)
                }
                "register" => {
                    let sub_var_group = VariableGroup::default();

                    let var_name = value.as_str()?;
                    let mut var_group_info = VariableGroupInfo::default();
                    var_group_info
                        .variable_locs
                        .push(crate::parser::variable::VariableInfo {
                            name: StringLoc::from(value, path),
                            value: "".to_string(),
                            source: source.clone(),
                        });

                    sub_var_group.insert(var_name.to_string(), var_group_info);
                    Some(sub_var_group)
                }
                _ => None,
            };

            var_group.merge(sub_var_group.unwrap_or_default());
        }
    }

    Some(var_group)
}

pub fn parse_task_vars(
    content: &str,
    path: &Path,
    role_name: &str,
    role_path: &Path,
) -> Option<VariableGroup> {
    let docs = load_yvalue_from_str(content).ok()?;
    if docs.len() != 1 {
        return None;
    }
    let source = crate::parser::variable::VariableSource::from_role(role_name, role_path);
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

        let xs = xs.map(|vg| vg.to_print_list());
        ts.assert_output(&xs);
    }
}
