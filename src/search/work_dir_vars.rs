use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::parser::variable::VariableGroup;
use crate::parser::zuul::job::Job;
use crate::search::jobs::list_jobs;
use crate::search::report_print::print_var_info_list;

fn collect_ordered_jobs(work_dir: &Path, config_path: Option<PathBuf>) -> Vec<Rc<Job>> {
    let work_dir_str = work_dir.to_str().unwrap();
    let jobs = list_jobs(work_dir, config_path);

    let work_dir_job_names: Vec<String> = jobs
        .jobs()
        .iter()
        .filter(|job| job.name().path.starts_with(work_dir_str))
        .map(|job| job.name().value.to_string())
        .collect();

    jobs.gen_job_topo_order(&work_dir_job_names)
}

pub fn list_work_dir_vars(work_dir: &Path, config_path: Option<PathBuf>) -> VariableGroup {
    let jobs = collect_ordered_jobs(work_dir, config_path);
    let mut vg = VariableGroup::default();

    jobs.iter()
        .map(|job| job.vars().clone().into())
        .for_each(|sub_vg| vg.merge(sub_vg));

    vg
}

pub fn list_work_dir_vars_cli(work_dir: &Path, config_path: Option<PathBuf>) {
    let vg = list_work_dir_vars(work_dir, config_path);
    let vars = vg.to_print_list();
    print_var_info_list(&vars);
}
