use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;

use bimap::BiMap;
use hashlink::LinkedHashMap;
use petgraph::algo::toposort;
use petgraph::algo::DfsSpace;
use petgraph::graph::DiGraph;

use crate::parser::common::StringLoc;
use crate::parser::zuul::job::Job;
use crate::parser::zuul::parse_zuul;
use crate::search::path::get_repo_dirs;
use crate::search::path::get_zuul_yaml_paths;

fn gather_jobs_by_name(jobs: Vec<Job>) -> LinkedHashMap<String, Vec<Job>> {
    let mut hs: LinkedHashMap<String, Vec<Job>> = LinkedHashMap::new();

    for j in jobs {
        let name = j.name().value.clone();

        match hs.get_mut(&name) {
            Some(value) => {
                value.push(j);
            }
            None => {
                hs.insert(name, vec![j]);
            }
        }
    }

    hs
}

fn collect_job_names(name: &str, jobs: &LinkedHashMap<String, Vec<Job>>) -> Vec<String> {
    let mut collect_names: HashSet<String> = HashSet::new();

    let mut search_names: VecDeque<String> = VecDeque::new();
    if let Some(value) = jobs.get(name) {
        search_names.push_back(name.to_string());
    }

    while !search_names.is_empty() {
        let name = search_names.pop_front().unwrap();
        let parent_jobs = jobs.get(&name).unwrap();
        collect_names.insert(name);

        for job in parent_jobs {
            let new_name = &job.name().value;
            if !collect_names.contains(new_name) {
                search_names.push_back(new_name.clone());
            }
        }
    }

    collect_names.into_iter().collect()
}

fn get_job_hierarchy(name: &str, raw_jobs: Vec<Job>) -> Vec<StringLoc> {
    let jobs = gather_jobs_by_name(raw_jobs);
    let names = collect_job_names(name, &jobs);

    let mut g = DiGraph::<&String, ()>::new();
    // let mut node_map = BiMap::new();
    let raw_nodes = names
        .iter()
        .map(|name| (name, g.add_node(name)))
        .collect::<Vec<_>>();

    let mut node_map = BiMap::new();
    for (name, node_idx) in raw_nodes {
        node_map.insert(name, node_idx);
    }

    for name in &names {
        let serach_jobs = jobs.get(name).unwrap();
        for job in serach_jobs {
            if let Some(parent_job_name) = job.parent() {
                let parent_job_name = &parent_job_name.value;
                if jobs.contains_key(parent_job_name) {
                    g.add_edge(
                        *node_map.get_by_left(name).unwrap(),
                        *node_map.get_by_left(&parent_job_name).unwrap(),
                        (),
                    );
                }
            }
        }
    }

    let mut space = DfsSpace::default();
    let hs = toposort(&g, Some(&mut space)).unwrap();
    let hs = hs
        .into_iter()
        .map(|node_idx| *node_map.get_by_right(&node_idx).unwrap())
        .collect::<Vec<_>>();

    let mut ys = Vec::new();
    for h in hs {
        let hierarchy_jobs = jobs.get(h).unwrap();
        for job in hierarchy_jobs {
            ys.push(job.name().clone());
        }
    }

    ys
}

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
