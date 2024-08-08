use std::collections::HashMap;
use std::path::Path;

use hashlink::LinkedHashMap;

use super::tasks::parse_task_vars_internal;
use crate::parser::var_table::{
    merge_var_group, parse_var_group, parse_var_group_from_hash, VarGroup, VariableSource,
};
use crate::parser::yaml::{load_yvalue_from_str, YValue};

fn parse_playbook_role_vars_internal(
    value: &YValue,
    path: &Path,
    source: &VariableSource,
) -> Option<VarGroup> {
    let mut var_group = HashMap::new();
    let role_tasks = value.as_vec()?;

    for raw_role_task in role_tasks {
        let role_task = raw_role_task.as_hash()?;

        let is_vars_attr_exist = role_task
            .keys()
            .map_while(|x| x.as_str())
            .find(|x| *x == "vars");

        let sub_var_group = match is_vars_attr_exist {
            Some(_) => {
                for (key, value) in role_task {
                    let key_name = key.as_str()?;
                    if key_name == "vars" {
                        return parse_var_group(value, path, "playbook", source).ok();
                    }
                }
                None
            }
            _ => {
                let vars = role_task
                    .iter()
                    .filter(|(key, _)| {
                        if let Some(key_name) = key.as_str() {
                            !matches!(key_name, "role" | "vars" | "tags")
                        } else {
                            true
                        }
                    })
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<LinkedHashMap<_, _>>();

                parse_var_group_from_hash(&vars, path, "playbook", source).ok()
            }
        };

        var_group = merge_var_group(var_group, sub_var_group.unwrap_or_default());
    }

    Some(var_group)
}

pub fn parse_playbook_vars(content: &str, path: &Path, _: &str, _: &Path) -> Option<VarGroup> {
    let source = VariableSource::from_playbook(path);
    let docs = load_yvalue_from_str(content).ok()?;
    let mut var_group = HashMap::new();

    for doc in docs {
        let xs = doc.as_vec()?;
        for x in xs {
            let playbook = x.as_hash()?;
            for (key, value) in playbook {
                let key_name = key.as_str()?;
                let sub_var_group = match key_name {
                    "vars" => parse_var_group(value, path, "playbook", &source).ok(),
                    "tasks" | "pre_tasks" | "post_tasks" => {
                        parse_task_vars_internal(value, path, "playbook", &source)
                    }
                    "roles" => parse_playbook_role_vars_internal(value, path, &source),
                    _ => None,
                };

                var_group = merge_var_group(var_group, sub_var_group.unwrap_or_default());
            }
        }
    }

    Some(var_group)
}
