use std::ops::{Deref, DerefMut};
use std::path::Path;

use hashlink::LinkedHashMap;
use interner::global::GlobalString;

use super::VariableSource;
use crate::parser::common::{parse_string_value, StringLoc, ZuulParseError};
use crate::parser::yaml::{YValue, YValueYaml};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct VariableTable(pub LinkedHashMap<GlobalString, Variable>);

impl Deref for VariableTable {
    type Target = LinkedHashMap<GlobalString, Variable>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VariableTable {
    pub fn parse_map(
        values: &LinkedHashMap<YValue, YValue>,
        path: &Path,
        field_name: &str,
        source: &VariableSource,
    ) -> Result<VariableTable, ZuulParseError> {
        let mut vs = VariableTable::default();
        for (key, value) in values {
            let key = parse_string_value(key, path, field_name)?;
            let value = Value::from_yvalue(value, path, key.as_str(), source)?;
            vs.insert(
                key.value.clone(),
                Variable {
                    name: key,
                    value,
                    source: source.clone(),
                },
            );
        }

        Ok(vs)
    }

    pub fn parse_yaml(
        values: &YValue,
        path: &Path,
        field_name: &str,
        source: &VariableSource,
    ) -> Result<VariableTable, ZuulParseError> {
        let values = values.as_hash().ok_or(ZuulParseError::from(
            format!("Failed to parse the value of {}", field_name)
                .to_string()
                .as_str(),
            values,
            path,
        ))?;

        Self::parse_map(values, path, field_name, source)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Variable {
    pub name: StringLoc,
    pub value: Value,
    pub source: VariableSource,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
pub enum Value {
    #[default]
    Null,
    Integer(i64),
    Boolean(bool),
    Real(String),
    String(String),
    Array(Vec<Self>),
    Hash(VariableTable),
}

impl Value {
    pub fn to_show_value(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Integer(v) => v.to_string(),
            Value::Boolean(v) => v.to_string(),
            Value::Real(v) => v.clone(),
            Value::String(v) => v.clone(),
            Value::Array(v) => {
                let s = v
                    .iter()
                    .map(|x| x.to_show_value())
                    .collect::<Vec<_>>()
                    .join(", ");
                ["[", &s, "]"].join("")
            }
            Value::Hash(v) => format!("{:?}", v),
        }
    }

    pub fn from_yvalue(
        value: &YValue,
        path: &Path,
        field_name: &str,
        source: &VariableSource,
    ) -> Result<Value, ZuulParseError> {
        Ok(match value.value() {
            YValueYaml::Real(v) => Value::Real(v.clone()),
            YValueYaml::Integer(v) => Value::Integer(*v),
            YValueYaml::String(v) => Value::String(v.clone()),
            YValueYaml::Boolean(v) => Value::Boolean(*v),
            YValueYaml::Array(vs) => {
                let mut xs = Vec::new();
                for v in vs {
                    xs.push(Value::from_yvalue(v, path, field_name, source)?);
                }
                Value::Array(xs)
            }
            YValueYaml::Hash(vs) => {
                let mut xs = VariableTable::default();
                for (key, value) in vs {
                    let key = parse_string_value(key, path, field_name)?;
                    let value = Value::from_yvalue(value, path, key.as_str(), source)?;
                    xs.insert(
                        key.value.clone(),
                        Variable {
                            name: key,
                            value,
                            source: source.clone(),
                        },
                    );
                }
                Value::Hash(xs)
            }
            YValueYaml::Null => Value::Null,
            YValueYaml::Alias(_) => unreachable!(),
            YValueYaml::BadValue => unreachable!(),
        })
    }
}
