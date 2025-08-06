use std::path::{Path, PathBuf};

use crate::parser::variable::VariableGroup;
use crate::search::jobs::ZuulJobs;
use crate::search::report_print::print_var_info_list;

use super::jobs::list_jobs_action_cli;

pub fn list_job_vars(name: &str, zuul_jobs: &ZuulJobs) -> VariableGroup {
    let jobs = zuul_jobs.get_job_hierarchy(name);
    let mut vg = VariableGroup::default();

    jobs.iter()
        .map(|job| job.vars().clone().into())
        .for_each(|sub_vg| vg.add(sub_vg));

    vg
}

pub fn list_jobs_vars_cli(job_name: String, work_dir: &Path, config_path: Option<PathBuf>) {
    list_jobs_action_cli(work_dir, config_path, |zuul_jobs| {
        let vg = list_job_vars(&job_name, &zuul_jobs);
        let vars = vg.to_print_list();

        print_var_info_list(&vars);
    });
}
