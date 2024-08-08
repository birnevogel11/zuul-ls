use std::path::PathBuf;
use std::rc::Rc;

use crate::parser::zuul::job::Job;
use crate::parser::zuul::parse_zuul;
use crate::search::path::get_repo_dirs;
use crate::search::path::get_zuul_yaml_paths;

pub fn list_jobs(yaml_paths: &[Rc<PathBuf>]) -> Vec<Job> {
    parse_zuul(yaml_paths).into_jobs()
}

pub fn list_jobs_cli(
    _search_key: Option<String>,
    work_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
) {
    let repo_dirs = get_repo_dirs(work_dir, config_path);
    let yaml_paths = get_zuul_yaml_paths(&repo_dirs);
    let jobs = list_jobs(&yaml_paths);

    for job in jobs {
        let name = job.name();
        println!(
            "{:?} {:?}:{:?}:{:?}",
            name.value, name.path, name.line, name.col
        );
    }
}
