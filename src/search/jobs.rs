use std::path::PathBuf;
use std::rc::Rc;

use crate::parser::zuul::job::Job;
use crate::parser::zuul::parse_zuul;
use crate::search::path::get_repo_dirs;
use crate::search::path::get_zuul_yaml_paths;

pub fn list_jobs_from_paths(yaml_paths: &[Rc<PathBuf>]) -> Vec<Job> {
    parse_zuul(yaml_paths).into_jobs()
}

pub fn list_jobs_from_cli(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) -> Vec<Job> {
    let repo_dirs = get_repo_dirs(work_dir, config_path);
    let yaml_paths = get_zuul_yaml_paths(&repo_dirs);
    list_jobs_from_paths(&yaml_paths)
}

pub fn list_jobs_cli(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) {
    let jobs = list_jobs_from_cli(work_dir, config_path);
    for job in jobs {
        let name = job.name();
        println!(
            "{} {}:{}:{}",
            name.value,
            name.path.to_str().unwrap(),
            name.line,
            name.col
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_key_test::TestFiles;

    #[test]
    fn test_list_jobs() {
        let ts = TestFiles::new("list_job_0.yaml");
        let paths = vec![Rc::new(ts.input_path.clone())];
        let jobs = list_jobs_from_paths(&paths);

        ts.assert_output(&jobs);
    }
}
