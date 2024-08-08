use hashlink::LinkedHashMap;
use std::collections::HashMap;
use std::path::Path;

use interner::global::{GlobalPath, GlobalString};

use crate::parser::common::{
    parse_string_value, StringLoc, ZuulParseError, PATH_POOL, STRING_POOL,
};
use crate::parser::yaml::{YValue, YValueYaml};

pub type VarTable = LinkedHashMap<StringLoc, VarValue>;
pub type VarGroup = HashMap<String, Vec<VariableInfo>>;

pub fn parse_var_table_from_hash(
    values: &LinkedHashMap<YValue, YValue>,
    path: &Path,
    field_name: &str,
) -> Result<VarTable, ZuulParseError> {
    let mut vs = VarTable::new();
    for (key, value) in values {
        let key = parse_string_value(key, path, field_name)?;
        let value = VarValue::from_yvalue(value, path, key.as_str())?;
        vs.insert(key, value);
    }

    Ok(vs)
}

pub fn parse_var_table(
    values: &YValue,
    path: &Path,
    field_name: &str,
) -> Result<VarTable, ZuulParseError> {
    let values = values.as_hash().ok_or(ZuulParseError::from(
        format!("Failed to parse the value of {}", field_name)
            .to_string()
            .as_str(),
        values,
        path,
    ))?;

    parse_var_table_from_hash(values, path, field_name)
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum VarValue {
    Null,
    Integer(i64),
    Boolean(bool),
    Real(String),
    String(String),
    Array(Vec<Self>),
    Hash(VarTable),
}

impl VarValue {
    pub fn to_show_value(&self) -> String {
        match self {
            VarValue::Null => "null".to_string(),
            VarValue::Integer(v) => v.to_string(),
            VarValue::Boolean(v) => v.to_string(),
            VarValue::Real(v) => v.clone(),
            VarValue::String(v) => v.clone(),
            VarValue::Array(v) => {
                let s = v
                    .iter()
                    .map(|x| x.to_show_value())
                    .collect::<Vec<_>>()
                    .join(", ");
                ["[", &s, "]"].join("")
            }
            VarValue::Hash(v) => format!("{:?}", v),
        }
    }

    pub fn from_yvalue(
        value: &YValue,
        path: &Path,
        field_name: &str,
    ) -> Result<VarValue, ZuulParseError> {
        Ok(match value.value() {
            YValueYaml::Real(v) => VarValue::Real(v.clone()),
            YValueYaml::Integer(v) => VarValue::Integer(*v),
            YValueYaml::String(v) => VarValue::String(v.clone()),
            YValueYaml::Boolean(v) => VarValue::Boolean(*v),
            YValueYaml::Array(vs) => {
                let mut xs = Vec::new();
                for v in vs {
                    xs.push(VarValue::from_yvalue(v, path, field_name)?);
                }
                VarValue::Array(xs)
            }
            YValueYaml::Hash(vs) => {
                let mut xs = VarTable::new();
                for (key, value) in vs {
                    let key = parse_string_value(key, path, field_name)?;
                    let value = VarValue::from_yvalue(value, path, key.as_str())?;
                    xs.insert(key, value);
                }
                VarValue::Hash(xs)
            }
            YValueYaml::Null => VarValue::Null,
            YValueYaml::Alias(_) => unreachable!(),
            YValueYaml::BadValue => unreachable!(),
        })
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum VariableSource {
    Unknown,
    Job(StringLoc),
    Role {
        name: GlobalString,
        path: GlobalPath,
    },
    Playbook(GlobalPath),
}

impl Default for VariableSource {
    fn default() -> Self {
        Self::Unknown
    }
}

impl VariableSource {
    pub fn from_role(name: &str, path: &Path) -> Self {
        Self::Role {
            name: STRING_POOL.get(name),
            path: PATH_POOL.get(path),
        }
    }

    pub fn from_playbook(path: &Path) -> Self {
        Self::Playbook(PATH_POOL.get(path))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct VariableInfo {
    pub name: StringLoc,
    pub value: String,
    pub source: VariableSource,
}

pub fn collect_variables(
    name_prefix: &str,
    var_table: &VarTable,
    source: &VariableSource,
) -> HashMap<String, VariableInfo> {
    let mut vs = HashMap::new();

    for (var, value) in var_table {
        let mut var_name = name_prefix.to_string();
        if !var_name.is_empty() {
            var_name.push('.');
        }
        var_name.push_str(&var.value);

        if !vs.contains_key(&var_name) {
            match value {
                VarValue::Hash(value) => {
                    let nested_vs = collect_variables(&var_name, value, source)
                        .into_iter()
                        .filter(|(x, _)| !vs.contains_key(x))
                        .collect::<HashMap<_, _>>();
                    vs.extend(nested_vs.into_iter());
                }
                _ => {
                    vs.insert(
                        var_name.clone(),
                        VariableInfo {
                            name: var.clone_loc(var_name),
                            source: source.clone(),
                            value: value.to_show_value(),
                        },
                    );
                }
            }
        }
    }
    vs
}

pub fn group_variables(group_vars: VarGroup, vars: HashMap<String, VariableInfo>) -> VarGroup {
    let mut group_vars = group_vars;

    vars.into_iter()
        .for_each(|(key, var_info)| match group_vars.get_mut(&key) {
            Some(info) => {
                info.push(var_info);
            }
            None => {
                group_vars.insert(key, vec![var_info]);
            }
        });

    group_vars
}

pub fn merge_var_group(xs: VarGroup, ys: VarGroup) -> VarGroup {
    let mut xs = xs;

    ys.into_iter()
        .for_each(|(key, var_info)| match xs.get_mut(&key) {
            Some(info) => {
                info.extend(var_info);
            }
            None => {
                xs.insert(key, var_info);
            }
        });

    xs
}
