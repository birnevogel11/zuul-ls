use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bimap::BiMap;
use hashlink::LinkedHashMap;
use log::debug;
use petgraph::algo::{toposort, DfsSpace};
use petgraph::graph::{DiGraph, Graph, NodeIndex};

use crate::parser::common::StringLoc;
use crate::parser::zuul::job::Job;
use crate::parser::zuul::parse_zuul;
use crate::search::path::get_zuul_yaml_paths_cwd;
use crate::search::path::to_path;
use crate::search::report_print::print_string_locs;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct ZuulJobs {
    jobs: Vec<Rc<Job>>,
    name_jobs: LinkedHashMap<String, Vec<Rc<Job>>>,
}

impl ZuulJobs {
    pub fn from_paths(yaml_paths: &[Rc<PathBuf>]) -> ZuulJobs {
        let jobs = parse_zuul(yaml_paths)
            .into_jobs()
            .into_iter()
            .map(Rc::new)
            .collect();
        let name_jobs = ZuulJobs::gather_jobs_by_name(&jobs);

        ZuulJobs { jobs, name_jobs }
    }

    pub fn from_raw_input(work_dir: &Path, config_path: Option<PathBuf>) -> ZuulJobs {
        let yaml_paths = get_zuul_yaml_paths_cwd(work_dir, config_path);
        let jobs = ZuulJobs::from_paths(&yaml_paths);

        debug!("jobs: {:#?}", jobs);

        jobs
    }

    pub fn from_parsed_jobs(parsed_jobs: Vec<Job>) -> ZuulJobs {
        let jobs = parsed_jobs.into_iter().map(Rc::new).collect();
        let name_jobs = ZuulJobs::gather_jobs_by_name(&jobs);

        ZuulJobs { jobs, name_jobs }
    }

    pub fn get_job_hierarchy(&self, name: &str) -> Vec<Rc<Job>> {
        // Try to support multiple inheritances ...
        let input_names = vec![name.to_string()];
        let names = ZuulJobs::collect_job_names(&input_names, &self.name_jobs);

        self.gen_job_topo_order(&names)
    }

    pub fn gen_job_topo_order(&self, input_job_names: &Vec<String>) -> Vec<Rc<Job>> {
        let job_names = ZuulJobs::collect_job_names(input_job_names, &self.name_jobs);

        // Create a di-graph and node mapping
        let (g, node_map) = ZuulJobs::create_job_graph(&job_names, &self.name_jobs);

        // Get the job hierarchy from the topological order of jobs.
        ZuulJobs::visit_job_graph(g, node_map, &self.name_jobs)
    }

    pub fn jobs(&self) -> &Vec<Rc<Job>> {
        &self.jobs
    }

    pub fn name_jobs(&self) -> &LinkedHashMap<String, Vec<Rc<Job>>> {
        &self.name_jobs
    }

    fn create_job_graph<'a>(
        names: &'a Vec<String>,
        jobs: &'a LinkedHashMap<String, Vec<Rc<Job>>>,
    ) -> (Graph<&'a String, ()>, BiMap<&'a String, NodeIndex>) {
        let mut g = DiGraph::<&String, ()>::new();
        let raw_nodes = names
            .iter()
            .map(|name| (name, g.add_node(name)))
            .collect::<Vec<_>>();

        let mut node_map = BiMap::new();
        for (name, node_idx) in raw_nodes {
            node_map.insert(name, node_idx);
        }

        for name in names {
            let search_jobs = jobs.get(name).unwrap();
            for job in search_jobs {
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
        (g, node_map)
    }

    fn visit_job_graph(
        g: Graph<&String, ()>,
        node_map: BiMap<&String, NodeIndex>,
        jobs: &LinkedHashMap<String, Vec<Rc<Job>>>,
    ) -> Vec<Rc<Job>> {
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
                ys.push(job.clone());
            }
        }

        ys
    }

    fn collect_job_names(
        names: &Vec<String>,
        jobs: &LinkedHashMap<String, Vec<Rc<Job>>>,
    ) -> Vec<String> {
        let mut collect_names: HashSet<String> = HashSet::new();

        let mut search_names: VecDeque<String> = VecDeque::new();
        for name in names {
            if jobs.contains_key(name) {
                search_names.push_back(name.to_string());
            }
        }

        while !search_names.is_empty() {
            let name = search_names.pop_front().unwrap();
            let parent_jobs = jobs.get(&name).unwrap();
            collect_names.insert(name);

            for job in parent_jobs {
                if let Some(parent) = &job.parent() {
                    let new_name = parent.value.clone();

                    if jobs.contains_key(&new_name) && !collect_names.contains(&new_name) {
                        search_names.push_back(new_name.clone());
                    }
                }
            }
        }

        collect_names.into_iter().collect()
    }

    fn gather_jobs_by_name(jobs: &Vec<Rc<Job>>) -> LinkedHashMap<String, Vec<Rc<Job>>> {
        let mut hs: LinkedHashMap<String, Vec<Rc<Job>>> = LinkedHashMap::new();

        for j in jobs {
            let name = j.name().value.clone();

            match hs.get_mut(&name) {
                Some(value) => {
                    value.push(j.clone());
                }
                None => {
                    hs.insert(name, vec![j.clone()]);
                }
            }
        }

        hs
    }
}

pub fn list_jobs_hierarchy_names_cli(
    job_name: String,
    work_dir: &Path,
    config_path: Option<PathBuf>,
) {
    let job_names = ZuulJobs::from_raw_input(work_dir, config_path)
        .get_job_hierarchy(&job_name)
        .iter()
        .map(|x| x.name().clone())
        .collect::<Vec<_>>();

    print_string_locs(&job_names)
}

pub fn list_jobs(work_dir: &Path, config_path: Option<PathBuf>) -> ZuulJobs {
    ZuulJobs::from_raw_input(work_dir, config_path)
}

pub fn list_jobs_cli(work_dir: &Path, config_path: Option<PathBuf>, is_local: bool) {
    let zuul_jobs = list_jobs(work_dir, config_path);

    let mut locs: Vec<StringLoc> = zuul_jobs.jobs().iter().map(|x| x.name().clone()).collect();
    if is_local {
        let sw = to_path(work_dir.to_str().unwrap());
        let sw = sw.to_str().unwrap();

        locs.retain(|x| {
            let s = to_path(x.path.to_str().unwrap());
            s.to_str().unwrap().starts_with(sw)
        });
    }
    let locs = locs;

    print_string_locs(&locs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_key_test::TestFiles;

    #[test]
    fn test_list_jobs() {
        let ts = TestFiles::new("list_job_0.yaml");
        let paths = vec![Rc::new(ts.input_path.clone())];
        let zuul_jobs = ZuulJobs::from_paths(&paths);
        let jobs = zuul_jobs.jobs();

        ts.assert_output(&jobs);
    }
}
