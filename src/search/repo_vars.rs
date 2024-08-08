// use std::path::PathBuf;
//
// use crate::search::jobs::list_jobs_from_cli;
//
// pub fn list_work_dir_vars_from_cli(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) {
//     let jobs = list_jobs_from_cli(work_dir.clone(), config_path);
//     let work_dir = work_dir.unwrap().to_str().unwrap();
//     for job in jobs.jobs() {
//         let job_path = job.name().path.to_str().unwrap();
//         if job_path.starts_with(work_dir) {}
//     }
// }
