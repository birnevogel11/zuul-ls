use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::common::parse_string_value;
use crate::parser::common::StringLoc;
use crate::parser::common::ZuulParse;
use crate::parser::common::ZuulParseError;
use crate::parser::yaml::{YValue, YValueYaml};
use crate::search::path::retrieve_repo_path;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum VarValue {
    Null,
    Integer(i64),
    Boolean(bool),
    Real(String),
    String(String),
    Array(Vec<VarValue>),
    Hash(VarTable),
}

pub type VarTable = LinkedHashMap<StringLoc, VarValue>;

fn parse_value(
    value: &YValue,
    path: &Rc<PathBuf>,
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
                xs.push(parse_value(v, path, field_name)?);
            }
            VarValue::Array(xs)
        }
        YValueYaml::Hash(vs) => {
            let mut xs = VarTable::new();
            for (key, value) in vs {
                let key = parse_string_value(key, path, field_name)?;
                let value = parse_value(value, path, key.as_str())?;
                xs.insert(key, value);
            }
            VarValue::Hash(xs)
        }
        YValueYaml::Null => VarValue::Null,
        YValueYaml::Alias(_) => unreachable!(),
        YValueYaml::BadValue => unreachable!(),
    })
}

fn parse_variables(
    values: &YValue,
    path: &Rc<PathBuf>,
    field_name: &str,
) -> Result<VarTable, ZuulParseError> {
    if let Some(values) = values.as_hash() {
        let mut vs = VarTable::new();
        for (key, value) in values {
            let key = parse_string_value(key, path, field_name)?;
            let value = parse_value(value, path, key.as_str())?;
            vs.insert(key, value);
        }

        Ok(vs)
    } else {
        Err(ZuulParseError::from(
            format!("Failed to parse the value of {}", field_name)
                .to_string()
                .as_str(),
            values,
            path,
        ))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct Job {
    name: StringLoc,
    description: Option<StringLoc>,
    parent: Option<StringLoc>,
    pre_run_playbooks: Vec<(StringLoc, PathBuf)>,
    run_playbooks: Vec<(StringLoc, PathBuf)>,
    post_run_playbooks: Vec<(StringLoc, PathBuf)>,
    clean_run_playbooks: Vec<(StringLoc, PathBuf)>,
    vars: VarTable,
}

impl Job {
    pub fn name(&self) -> &StringLoc {
        &self.name
    }

    pub fn parent(&self) -> &Option<StringLoc> {
        &self.parent
    }

    pub fn vars(&self) -> &VarTable {
        &self.vars
    }

    fn parse_playbook_list_item(
        value: &YValue,
        path: &Rc<PathBuf>,
        field_name: &str,
    ) -> Result<StringLoc, ZuulParseError> {
        if let Ok(value) = parse_string_value(value, path, field_name) {
            Ok(value)
        } else {
            let err = Err(ZuulParseError::from(
                format!("Failed to parse the value of {}", field_name)
                    .to_string()
                    .as_str(),
                value,
                path,
            ));

            match value.value() {
                YValueYaml::Hash(vs) => {
                    for (key, value) in vs {
                        if let (Some(key), Some(_)) = (key.as_str(), value.as_str()) {
                            if key == "name" {
                                return Ok(StringLoc::from(value, path));
                            }
                        }
                    }
                    err
                }
                _ => err,
            }
        }
    }

    fn parse_playbooks(
        value: &YValue,
        path: &Rc<PathBuf>,
        field_name: &str,
    ) -> Result<Vec<(StringLoc, PathBuf)>, ZuulParseError> {
        let mut values = Vec::new();

        if let Ok(value) = parse_string_value(value, path, field_name) {
            values.push(value);
        } else if let Some(vs) = value.as_vec() {
            for value in vs {
                values.push(Job::parse_playbook_list_item(value, path, field_name)?)
            }
        }

        Ok(values
            .into_iter()
            .map(|x| {
                (
                    retrieve_repo_path(path.to_str().unwrap()).join(x.as_str()),
                    x,
                )
            })
            .map(|x| (x.1, x.0))
            .collect())
    }
}

impl ZuulParse<Job> for Job {
    fn parse(
        xs: &LinkedHashMap<YValue, YValue>,
        path: &Rc<PathBuf>,
    ) -> Result<Job, ZuulParseError> {
        let mut name = StringLoc::default();
        let mut description: Option<StringLoc> = None;
        let mut parent: Option<StringLoc> = None;
        let mut pre_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut post_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut clean_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut vars: VarTable = VarTable::new();

        for (key, value) in xs {
            match key.as_str() {
                Some(key) => match key {
                    "name" => {
                        name = parse_string_value(value, path, "name")?;
                    }
                    "parent" => {
                        parent = Some(parse_string_value(value, path, "parent")?);
                    }
                    "description" => {
                        description = Some(parse_string_value(value, path, "description")?);
                    }
                    "pre-run" => {
                        pre_run_playbooks = Job::parse_playbooks(value, path, "pre-run")?;
                    }
                    "run" => {
                        run_playbooks = Job::parse_playbooks(value, path, "run")?;
                    }
                    "post-run" => {
                        post_run_playbooks = Job::parse_playbooks(value, path, "post-run")?;
                    }
                    "clean-run" => {
                        clean_run_playbooks = Job::parse_playbooks(value, path, "clean-run")?;
                    }
                    "vars" => {
                        vars = parse_variables(value, path, "vars")?;
                    }
                    "roles" => todo!(),
                    _ => {}
                },
                None => {
                    return Err(ZuulParseError::from("Failed to parse key", key, path));
                }
            }
        }

        Ok(Job {
            name,
            description,
            parent,
            pre_run_playbooks,
            run_playbooks,
            post_run_playbooks,
            clean_run_playbooks,
            vars,
        })
    }
}