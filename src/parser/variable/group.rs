use std::ops::{Deref, DerefMut};

use dashmap::DashMap;

use crate::parser::common::StringLoc;

use super::table::Value;
use super::table::VariableTable;
use super::VariableSource;

pub const ARRAY_INDEX_KEY: &str = "ArRaY_InDeX";

#[derive(Clone, Debug, Default)]
pub struct VariableGroup(DashMap<String, VariableGroupInfo>);

impl Deref for VariableGroup {
    type Target = DashMap<String, VariableGroupInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct VariableInfo {
    pub name: StringLoc,
    pub value: String,
    pub source: VariableSource,
}

#[derive(Clone, Debug, Default)]
pub struct VariableGroupInfo {
    pub variable_locs: Vec<VariableInfo>,
    pub members: VariableGroup,
}

fn from_var_table(var_group: &mut VariableGroup, var_table: &VariableTable) {
    var_table.0.iter().for_each(|(key, value)| {
        let key = key.to_string();
        if var_group.get(&key).is_none() {
            var_group.insert(key.clone(), VariableGroupInfo::default());
        }

        let mut vgi = var_group.get_mut(&key).unwrap();
        vgi.variable_locs.push(VariableInfo {
            name: value.name.clone(),
            value: value.value.to_show_value(),
            source: value.source.clone(),
        });

        match &value.value {
            Value::Hash(sub_var_table) => {
                from_var_table(&mut vgi.members, sub_var_table);
            }
            Value::Array(xs) => {
                if !xs.is_empty() {
                    let value = &xs[0];
                    if let Value::Hash(sub_var_table) = value {
                        let mut sub_vgi = VariableGroupInfo::default();
                        from_var_table(&mut sub_vgi.members, sub_var_table);
                        vgi.members.insert(ARRAY_INDEX_KEY.to_string(), sub_vgi);
                    }
                }
            }
            _ => {}
        }
    });
}

fn merge_var_group(
    var_group: &mut VariableGroup,
    new_var_group: VariableGroup,
    is_merge_same_name: bool,
) {
    new_var_group
        .0
        .into_iter()
        .for_each(|(key, value)| match var_group.get_mut(&key) {
            Some(mut curr_value) => {
                if is_merge_same_name {
                    curr_value.variable_locs.extend(value.variable_locs);
                }
                merge_var_group(&mut curr_value.members, value.members, is_merge_same_name);
            }
            None => {
                var_group.insert(key, value);
            }
        })
}

impl From<VariableTable> for VariableGroup {
    fn from(var_table: VariableTable) -> Self {
        let mut var_group = Self::default();
        from_var_table(&mut var_group, &var_table);
        var_group
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct VariablePrintInfo {
    pub name: StringLoc,
    pub value: String,
    pub source: VariableSource,
}

fn to_var_print_info_list(name_prefix: &str, var_group: &VariableGroup) -> Vec<VariablePrintInfo> {
    let mut vpi = Vec::new();

    var_group.iter().for_each(|entry| {
        let key = entry.key();
        let value = entry.value();

        let mut var_name = name_prefix.to_string();
        if !var_name.is_empty() {
            var_name.push('.');
        }
        var_name.push_str(key.as_ref());

        value.variable_locs.iter().for_each(|vi| {
            let name = vi.name.clone_loc(&var_name);
            vpi.push(VariablePrintInfo {
                name,
                value: vi.value.clone(),
                source: vi.source.clone(),
            })
        });

        vpi.extend(to_var_print_info_list(&var_name, &value.members));
    });

    vpi
}

impl VariableGroup {
    /// Merge the variable group. If there are the same variables, extend
    /// the locations and members recursively.
    pub fn merge(&mut self, var_group: VariableGroup) {
        merge_var_group(self, var_group, true);
    }

    /// Merge the variable group. If there are the same variables, skip
    /// the locations and members.
    pub fn add(&mut self, var_group: VariableGroup) {
        merge_var_group(self, var_group, false);
    }

    /// Flatten the variable group to a list of human-readable variable information
    pub fn to_print_list(&self) -> Vec<VariablePrintInfo> {
        let mut vs = to_var_print_info_list("", self);
        vs.sort();
        vs
    }
}
