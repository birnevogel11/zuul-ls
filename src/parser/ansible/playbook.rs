use std::path::Path;

use hashlink::LinkedHashMap;

use super::tasks::parse_task_vars_internal;
use crate::parser::variable::{VariableGroup, VariableTable};
use crate::parser::yaml::{load_yvalue_from_str, YValue};

fn parse_playbook_role_vars_internal(
    value: &YValue,
    path: &Path,
    source: &crate::parser::variable::VariableSource,
) -> Option<VariableGroup> {
    let mut var_group = VariableGroup::default();
    let role_tasks = value.as_vec()?;

    for raw_role_task in role_tasks {
        let role_task = raw_role_task.as_hash()?;

        let is_vars_attr_exist = role_task
            .keys()
            .map_while(|x| x.as_str())
            .find(|x| *x == "vars");

        let sub_var_group: Option<VariableGroup> = match is_vars_attr_exist {
            Some(_) => {
                for (key, value) in role_task {
                    let key_name = key.as_str()?;
                    if key_name == "vars" {
                        return Some(
                            VariableTable::parse_yaml(value, path, "playbook", source)
                                .ok()?
                                .into(),
                        );
                    }
                }
                None
            }
            _ => {
                let vars = role_task
                    .iter()
                    .filter(|(key, _)| {
                        if let Some(key_name) = key.as_str() {
                            !matches!(
                                key_name,
                                "role" | "vars" | "tags" | "name" | "when" | "register"
                            )
                        } else {
                            true
                        }
                    })
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<LinkedHashMap<_, _>>();

                Some(
                    VariableTable::parse_map(&vars, path, "playbook", source)
                        .ok()?
                        .into(),
                )
            }
        };

        var_group.merge(sub_var_group.unwrap_or_default())
    }

    Some(var_group)
}

pub fn parse_playbook_vars(content: &str, path: &Path, _: &str, _: &Path) -> Option<VariableGroup> {
    let source = crate::parser::variable::VariableSource::from_playbook(path);
    let docs = load_yvalue_from_str(content).ok()?;
    let mut var_group = VariableGroup::default();

    for doc in docs {
        let xs = doc.as_vec()?;
        for x in xs {
            let playbook = x.as_hash()?;
            for (key, value) in playbook {
                let key_name = key.as_str()?;
                let sub_var_group: Option<VariableGroup> = match key_name {
                    "vars" | "set_fact" => Some(
                        VariableTable::parse_yaml(value, path, "playbook", &source)
                            .ok()?
                            .into(),
                    ),
                    "tasks" | "pre_tasks" | "post_tasks" => {
                        parse_task_vars_internal(value, path, "playbook", &source)
                    }
                    "roles" => parse_playbook_role_vars_internal(value, path, &source),
                    _ => None,
                };

                var_group.merge(sub_var_group.unwrap_or_default())
            }
        }
    }

    Some(var_group)
}
