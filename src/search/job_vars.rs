use std::path::{Path, PathBuf};
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::common::StringLoc;
use crate::parser::zuul::job::{VarTable, VarValue};
use crate::search::jobs::{print_var_info_list, VariableInfo, ZuulJobs};

fn collect_variables(
    name_prefix: &str,
    job_vars: &VarTable,
    job_name: &Rc<StringLoc>,
) -> LinkedHashMap<String, VariableInfo> {
    let mut vs = LinkedHashMap::new();

    for (job_var, value) in job_vars {
        let mut var_name = name_prefix.to_string();
        if !var_name.is_empty() {
            var_name.push('.');
        }
        var_name.push_str(&job_var.value);

        if !vs.contains_key(&var_name) {
            match value {
                VarValue::Hash(value) => {
                    let nested_vs = collect_variables(&var_name, value, job_name)
                        .into_iter()
                        .filter(|(x, _)| !vs.contains_key(x))
                        .collect::<LinkedHashMap<_, _>>();
                    vs.extend(nested_vs.into_iter());
                }
                _ => {
                    vs.insert(
                        var_name.clone(),
                        VariableInfo {
                            name: job_var.assign_value(var_name),
                            job_name: job_name.clone(),
                            value: value.to_show_value(),
                        },
                    );
                }
            }
        }
    }
    vs
}

pub fn list_job_vars(name: &str, zuul_jobs: &ZuulJobs) -> LinkedHashMap<String, VariableInfo> {
    let jobs = zuul_jobs.get_job_hierarchy(name);
    let mut vs = LinkedHashMap::new();

    for job in jobs {
        let job_name = Rc::new(job.name().clone());
        let ys = collect_variables("", job.vars(), &job_name)
            .into_iter()
            .filter(|(x, _)| !vs.contains_key(x))
            .collect::<LinkedHashMap<_, _>>();
        vs.extend(ys.into_iter());
    }

    vs
}

pub fn list_jobs_vars_cli(job_name: String, work_dir: &Path, config_path: Option<PathBuf>) {
    let zuul_jobs = ZuulJobs::from_raw_input(work_dir, config_path);
    let vars = list_job_vars(&job_name, &zuul_jobs);

    let vars = vars
        .into_iter()
        .map(|(_, var_info)| var_info)
        .collect::<Vec<_>>();
    print_var_info_list(vars);
}
