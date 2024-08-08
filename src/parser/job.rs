use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::common::parse_string_or_list_string;
use crate::parser::common::parse_string_value;
use crate::parser::common::StringLoc;
use crate::parser::common::ZuulParse;
use crate::parser::common::ZuulParseError;
use crate::parser::yaml::{load_yvalue, YValue, YValueYaml};
use crate::search::path::retrieve_repo_path;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct Job {
    job_name: StringLoc,
    parent: Option<StringLoc>,
    pre_run_playbooks: Vec<(StringLoc, PathBuf)>,
    run_playbooks: Vec<(StringLoc, PathBuf)>,
    post_run_playbooks: Vec<(StringLoc, PathBuf)>,
    clean_run_playbooks: Vec<(StringLoc, PathBuf)>,
}

impl Job {
    fn parse_playbooks(
        value: &YValue,
        path: &Rc<PathBuf>,
        field_name: &str,
    ) -> Result<Vec<(StringLoc, PathBuf)>, ZuulParseError> {
        let value = parse_string_or_list_string(value, path, field_name)?;
        Ok(value
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
        let mut job_name = StringLoc::default();
        let mut parent: Option<StringLoc> = None;
        let mut pre_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut post_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();
        let mut clean_run_playbooks: Vec<(StringLoc, PathBuf)> = Vec::new();

        for (key, value) in xs {
            match key.as_str() {
                Some(key) => match key {
                    "job" => {
                        let value = parse_string_value(value, path, "job")?;
                        job_name = value;
                    }
                    "parent" => {
                        let value = parse_string_value(value, path, "parent")?;
                        parent = Some(value);
                    }
                    "prerun" => {
                        let value = Job::parse_playbooks(value, path, "prerun")?;
                        pre_run_playbooks = value;
                    }
                    "run" => {
                        let value = Job::parse_playbooks(value, path, "run")?;
                        run_playbooks = value;
                    }
                    "postrun" => {
                        let value = Job::parse_playbooks(value, path, "postrun")?;
                        post_run_playbooks = value;
                    }
                    "cleanrun" => {
                        let value = Job::parse_playbooks(value, path, "cleanrun")?;
                        clean_run_playbooks = value;
                    }
                    _ => {}
                },
                None => {
                    return Err(ZuulParseError::from("Failed to parse key", key, path));
                }
            }
        }

        Ok(Job {
            job_name,
            parent,
            pre_run_playbooks,
            run_playbooks,
            post_run_playbooks,
            clean_run_playbooks,
        })
    }
}
