use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use interner::global::GlobalString;

use crate::parser::zuul::job::Job;

use super::jobs::ZuulJobs;
use super::work_dir_vars::collect_ordered_workdir_jobs;

#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct ZuulJobGraph {
    jobs: HashSet<GlobalString>,
    edges: HashSet<(GlobalString, GlobalString)>,
}

fn make_job_graph(jobs: &[Rc<Job>]) -> ZuulJobGraph {
    let mut g = ZuulJobGraph::default();
    jobs.iter().for_each(|job| {
        if let Some(parent) = job.parent() {
            let name = job.name();
            g.jobs.insert(name.value.clone());
            g.jobs.insert(parent.value.clone());
            g.edges.insert((name.value.clone(), parent.value.clone()));
        }
    });

    g
}

fn render_job_graph_plantuml(job_graph: &ZuulJobGraph) {
    let mut nodes = job_graph.jobs.iter().collect::<Vec<_>>();
    nodes.sort();

    let mut edges = job_graph.edges.iter().collect::<Vec<_>>();
    edges.sort();

    println!("@startuml");
    for node in &nodes {
        println!(r#"rectangle "{}""#, node);
    }

    for (c, p) in &edges {
        println!(r#""{}" -up->> "{}""#, c, p);
    }
    println!("@enduml");
}

pub fn make_job_graph_cli(work_dir: &Path, config_path: Option<PathBuf>) {
    let zuul_jobs = ZuulJobs::from_raw_input(work_dir, config_path);
    let jobs = collect_ordered_workdir_jobs(&zuul_jobs, work_dir);
    let job_graph = make_job_graph(&jobs);
    render_job_graph_plantuml(&job_graph);
}
