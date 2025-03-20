use std::path::Path;

use hashlink::LinkedHashMap;

use crate::parser::common::{
    parse_list_string_value, parse_string_value, StringLoc, ZuulParse, ZuulParseError,
};
use crate::parser::yaml::{YValue, YValueYaml};

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct ProjectTemplate {
    name: StringLoc,
    description: StringLoc,
    templates: Vec<StringLoc>,
    pipeline_jobs: LinkedHashMap<String, Vec<StringLoc>>,
}

impl ProjectTemplate {
    pub fn name(&self) -> &StringLoc {
        &self.name
    }

    pub fn description(&self) -> &StringLoc {
        &self.description
    }

    pub fn pipeline_jobs(&self) -> &LinkedHashMap<String, Vec<StringLoc>> {
        &self.pipeline_jobs
    }

    fn parse_pipeline_jobs(
        value: &YValue,
        path: &Path,
        field_name: &str,
    ) -> Result<Vec<StringLoc>, ZuulParseError> {
        let mut job_names: Vec<StringLoc> = Vec::new();

        if let YValueYaml::Array(vs) = value.value() {
            for v in vs {
                match v.value() {
                    YValueYaml::String(_) => {
                        job_names.push(StringLoc::from(v, path));
                    }
                    YValueYaml::Hash(vs) => {
                        if vs.len() == 1 {
                            for key in vs.keys() {
                                if let YValueYaml::String(_) = key.value() {
                                    job_names.push(StringLoc::from(key, path));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(ZuulParseError::from(
                            format!("Failed to parse the value of {}", field_name)
                                .to_string()
                                .as_str(),
                            v,
                            path,
                        ));
                    }
                }
            }
        }

        Ok(job_names)
    }

    fn parse_pipeline(
        value: &YValue,
        path: &Path,
        field_name: &str,
    ) -> Result<Vec<StringLoc>, ZuulParseError> {
        if let YValueYaml::Hash(vs) = value.value() {
            for (key, value) in vs {
                if let Some(key) = key.as_str() {
                    if key == "<<" {
                        return Self::parse_pipeline(value, path, field_name);
                    }
                    if key == "jobs" {
                        return Self::parse_pipeline_jobs(value, path, key);
                    }
                }
            }
        }
        Err(ZuulParseError::from(
            format!("Failed to parse the value of {}", field_name)
                .to_string()
                .as_str(),
            value,
            path,
        ))
    }
}

impl ZuulParse<ProjectTemplate> for ProjectTemplate {
    fn parse(
        xs: &LinkedHashMap<YValue, YValue>,
        path: &Path,
    ) -> Result<ProjectTemplate, ZuulParseError> {
        let mut name: Option<StringLoc> = None;
        let mut description: Option<StringLoc> = None;
        let mut pipeline_jobs: LinkedHashMap<String, Vec<StringLoc>> = LinkedHashMap::new();
        let mut templates: Vec<StringLoc> = Vec::new();

        for (key, value) in xs {
            if let Some(key) = key.as_str() {
                match key {
                    "name" => {
                        name = Some(parse_string_value(value, path, "name")?);
                    }
                    "description" => {
                        description = Some(parse_string_value(value, path, "description")?);
                    }
                    "templates" => match parse_list_string_value(value, path, "templates") {
                        Ok(names) => {
                            templates = names;
                        }
                        Err(e) => {
                            log::warn!("Failed to parse import-templates. Skip to parse it. path: {:?}: error: {:#?}. Skip to parse it", path, e);
                        }
                    },
                    "queue" | "vars" | "merge-mode" => {}
                    _ => {
                        let pipeline_name = key;
                        match ProjectTemplate::parse_pipeline(value, path, pipeline_name) {
                            Ok(job_names) => {
                                pipeline_jobs.insert(pipeline_name.to_string(), job_names);
                            }
                            Err(e) => {
                                log::warn!("Failed to parse pipeline. Skip to parse it. path: {:?}: error: {:#?}. Skip to parse it", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(ProjectTemplate {
            name: name.unwrap_or_default(),
            description: description.unwrap_or_default(),
            templates,
            pipeline_jobs,
        })
    }
}
