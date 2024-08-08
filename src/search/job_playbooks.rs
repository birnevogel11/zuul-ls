use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::parser::common::StringLoc;
use crate::path::shorten_path;
use crate::safe_println;
use crate::search::jobs::ZuulJobs;

#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct PlaybookInfo {
    name: StringLoc,
    path: PathBuf,
    job_name: Rc<String>,
}

#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct JobPlaybooks {
    pre_run: Vec<PlaybookInfo>,
    run: Vec<PlaybookInfo>,
    post_run: Vec<PlaybookInfo>,
}

fn append_playbooks(
    new_ps: &[(StringLoc, PathBuf)],
    job_name: &Rc<String>,
    ps: &mut Vec<PlaybookInfo>,
) {
    let mut new_ps = new_ps
        .iter()
        .map(|(name, path)| PlaybookInfo {
            name: name.clone(),
            path: path.clone(),
            job_name: job_name.clone(),
        })
        .collect::<Vec<_>>();

    ps.append(&mut new_ps);
}

fn show_playbooks(name: &str, pbs: &[PlaybookInfo]) {
    if pbs.is_empty() {
        return;
    }
    for pb in pbs {
        safe_println!(
            "{}\t{}\t{}",
            shorten_path(&pb.path).display(),
            name,
            pb.job_name
        );
    }
}

pub fn list_job_playbooks(name: &str, zuul_jobs: &ZuulJobs) -> JobPlaybooks {
    let jobs = zuul_jobs.get_job_hierarchy(name);
    let re_jobs = jobs.iter().rev().collect::<Vec<_>>();
    let mut jp = JobPlaybooks::default();

    for job in re_jobs {
        let job_name = Rc::new(job.name().value.to_string());

        for (new_ps, ps) in [
            (job.pre_run_playbooks(), &mut jp.pre_run),
            (job.run_playbooks(), &mut jp.run),
        ] {
            append_playbooks(new_ps, &job_name, ps);
        }
    }

    for job in jobs {
        let job_name = Rc::new(job.name().value.to_string());
        let (new_ps, ps) = (job.post_run_playbooks(), &mut jp.post_run);
        append_playbooks(new_ps, &job_name, ps);
    }

    jp
}

pub fn list_jobs_playbooks_cli(job_name: String, work_dir: &Path, config_path: Option<PathBuf>) {
    let zuul_jobs = ZuulJobs::from_raw_input(work_dir, config_path);
    let jps = list_job_playbooks(&job_name, &zuul_jobs);

    show_playbooks("pre-run", &jps.pre_run);
    show_playbooks("run", &jps.run);
    show_playbooks("post-run", &jps.post_run);
}
