use std::path::{Path, PathBuf};

use hashlink::LinkedHashMap;

use crate::parser::var_table::{collect_variables, VariableInfo, VariableSource};
use crate::search::jobs::ZuulJobs;
use crate::search::report_print::print_var_info_list;

pub fn list_job_vars(name: &str, zuul_jobs: &ZuulJobs) -> LinkedHashMap<String, VariableInfo> {
    let jobs = zuul_jobs.get_job_hierarchy(name);
    let mut vs = LinkedHashMap::new();

    for job in jobs {
        let job_name = job.name().clone();
        let ys = collect_variables("", job.vars(), &VariableSource::Job(job_name))
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
    print_var_info_list(&vars);
}
