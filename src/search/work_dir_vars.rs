use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::parser::common::StringLoc;
use crate::parser::zuul::job::{VarTable, VarValue};
use crate::search::job_vars::VariableInfo;
use crate::search::jobs::list_jobs;
use crate::search::report_print::print_var_info_list;

fn expand_vars(
    name_prefix: &str,
    job_vars: &VarTable,
    job_name: &Rc<StringLoc>,
) -> HashSet<VariableInfo> {
    let mut vs: HashSet<VariableInfo> = HashSet::new();
    for (job_var, value) in job_vars {
        let mut var_name = name_prefix.to_string();
        if !var_name.is_empty() {
            var_name.push('.');
        }
        var_name.push_str(&job_var.value);

        match value {
            VarValue::Hash(value) => {
                vs.extend(expand_vars(&var_name, value, job_name));
            }
            _ => {
                vs.insert(VariableInfo {
                    name: job_var.clone_loc(var_name),
                    job_name: job_name.clone(),
                    value: value.to_show_value(),
                });
            }
        }
    }

    vs
}

pub fn list_work_dir_vars(work_dir: &Path, config_path: Option<PathBuf>) -> HashSet<VariableInfo> {
    let work_dir_str = work_dir.to_str().unwrap();
    let jobs = list_jobs(work_dir, config_path);

    let work_dir_job_names: Vec<String> = jobs
        .jobs()
        .iter()
        .filter(|job| job.name().path.to_str().unwrap().starts_with(work_dir_str))
        .map(|job| job.name().value.clone())
        .collect();
    let ordered_jobs = jobs.gen_job_topo_order(&work_dir_job_names);

    let mut vars: HashSet<VariableInfo> = HashSet::new();
    for job in ordered_jobs {
        let job_name = Rc::new(job.name().clone());
        vars.extend(expand_vars("", job.vars(), &job_name));
    }

    vars
}

pub fn list_work_dir_vars_cli(work_dir: &Path, config_path: Option<PathBuf>) {
    let vars = list_work_dir_vars(work_dir, config_path);
    print_var_info_list(&vars.into_iter().collect::<Vec<_>>());
}
