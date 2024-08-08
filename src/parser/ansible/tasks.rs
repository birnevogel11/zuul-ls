use std::collections::HashMap;
use std::path::Path;

use crate::parser::common::StringLoc;
use crate::parser::var_table::{
    collect_variables, group_variables, parse_var_table, VarGroup, VariableInfo, VariableSource,
};
use crate::parser::yaml::{load_yvalue_from_str, YValue};

fn parse_task_vars_internal(
    value: &YValue,
    path: &Path,
    role_name: &str,
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
            if key_name == "set_fact" {
                let vt = parse_var_table(value, path, role_name).ok()?;
                let sub_var_info = collect_variables("", &vt, source)
                    .into_iter()
                    .filter(|(name, _)| name != "cachable")
                    .collect::<HashMap<_, _>>();
                var_group = group_variables(var_group, sub_var_info);
                // var_info.extend(sub_var_info);
            }
            if key_name == "block" {
                let sub_var_group = parse_task_vars_internal(value, path, role_name, source)?;
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
            if key_name == "register" {
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
