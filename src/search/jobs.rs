use std::path::PathBuf;

use crate::parser::zuul::job::Job;
use crate::parser::zuul::parse_zuul;
use crate::search::path::get_repo_dirs;
use crate::search::path::get_zuul_yaml_paths;

pub fn list_jobs(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) -> Vec<Job> {
    let repo_dirs = get_repo_dirs(work_dir, config_path);
    let yaml_paths = get_zuul_yaml_paths(&repo_dirs);

    parse_zuul(&yaml_paths)
        .into_iter()
        .map_while(|x| x.into_job())
        .collect::<Vec<_>>()
}

pub fn list_jobs_cli(
    search_key: Option<String>,
    work_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
) {
    let jobs = list_jobs(work_dir, config_path);
    for job in jobs {
        let name = job.name();
        println!(
            "{:?} {:?}:{:?}:{:?}",
            name.value, name.path, name.line, name.col
        );
    }
}
