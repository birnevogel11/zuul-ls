pub mod job;
pub mod nodeset;
pub mod pipeline;
pub mod project_template;
pub mod queue;
pub mod secret;

use std::path::Path;
use std::path::PathBuf;

use hashlink::LinkedHashMap;
use phf::phf_map;

use crate::parser::common::ZuulParse;
use crate::parser::yaml::{load_yvalue, YValue, YValueYaml};
use crate::parser::zuul::job::Job;
use crate::parser::zuul::nodeset::Nodeset;
use crate::parser::zuul::pipeline::Pipeline;
use crate::parser::zuul::project_template::ProjectTemplate;
use crate::parser::zuul::queue::Queue;
use crate::parser::zuul::secret::Secret;

#[derive(Clone)]
pub enum ZuulParseType {
    Job,
    ProjectTemplate,
    Project,
    Nodeset,
    Queue,
    Pipeline,
    Secret,
}

static ZUUL_PARSE_KEYWORDS: phf::Map<&'static str, ZuulParseType> = phf_map! {
    "job" => ZuulParseType::Job,
    "project-template" => ZuulParseType::ProjectTemplate,
    "project" => ZuulParseType::Project,
    "nodeset" => ZuulParseType::Nodeset,
    "queue" => ZuulParseType::Queue,
    "pipeline" => ZuulParseType::Pipeline,
    "secret" => ZuulParseType::Secret,
};

impl ZuulParseType {
    pub fn determine(key: &str) -> Option<ZuulParseType> {
        ZUUL_PARSE_KEYWORDS.get(key).cloned()
    }

    pub fn determine_old(xs: &LinkedHashMap<YValue, YValue>) -> Option<ZuulParseType> {
        for (key, _) in xs {
            if let Some(key) = key.as_str() {
                return ZUUL_PARSE_KEYWORDS.get(key).cloned();
            }
        }

        None
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub enum ZuulConfigParsedElement {
    Job(Job),
    ProjectTemplate(ProjectTemplate),
    Project(ProjectTemplate),
    Nodeset(Nodeset),
    Queue(Queue),
    Pipeline(Pipeline),
    Secret(Secret),
}

impl ZuulConfigParsedElement {
    fn retrieve_key_and_value(
        config: &YValue,
    ) -> Option<(ZuulParseType, &LinkedHashMap<YValue, YValue>)> {
        match config.as_hash() {
            Some(config) => {
                let (key, value) = config.into_iter().next()?;
                let key = key.as_str()?;
                let value = value.as_hash()?;
                let parse_type = ZuulParseType::determine(key)?;

                Some((parse_type, value))
            }
            None => None,
        }
    }

    pub fn parse(raw_config: &YValue, path: &Path) -> Option<ZuulConfigParsedElement> {
        match ZuulConfigParsedElement::retrieve_key_and_value(raw_config) {
            None => None,
            Some((parse_type, values)) => {
                let e = match parse_type {
                    ZuulParseType::Job => {
                        ZuulConfigParsedElement::Job(Job::parse(values, path).ok()?)
                    }
                    ZuulParseType::ProjectTemplate => ZuulConfigParsedElement::ProjectTemplate(
                        ProjectTemplate::parse(values, path).ok()?,
                    ),
                    ZuulParseType::Project => {
                        ZuulConfigParsedElement::Project(ProjectTemplate::parse(values, path).ok()?)
                    }
                    ZuulParseType::Nodeset => {
                        ZuulConfigParsedElement::Nodeset(Nodeset::parse(values, path).ok()?)
                    }
                    ZuulParseType::Queue => {
                        ZuulConfigParsedElement::Queue(Queue::parse(values, path).ok()?)
                    }
                    ZuulParseType::Pipeline => {
                        ZuulConfigParsedElement::Pipeline(Pipeline::parse(values, path).ok()?)
                    }
                    ZuulParseType::Secret => {
                        ZuulConfigParsedElement::Secret(Secret::parse(values, path).ok()?)
                    }
                };
                Some(e)
            }
        }
    }
}

macro_rules! define_as_ref (
    ($name:ident, $t:ty) => (
/// Get a reference to the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some(&$t)` with the `$t` contained. Otherwise,
/// return `None`.
#[must_use]
pub fn $name(&self) -> &Vec<$t> {
    &self.$name
}
    );
);

macro_rules! define_into (
    ($name:ident, $var:ident, $t:ty) => (
/// Get a reference to the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some(&$t)` with the `$t` contained. Otherwise,
/// return `None`.
#[must_use]
pub fn $name(self) -> Vec<$t> {
    self.$var
}
    );
);

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Default)]
pub struct ZuulConfigElements {
    jobs: Vec<Job>,
    project_templates: Vec<ProjectTemplate>,
    projects: Vec<ProjectTemplate>,
    nodesets: Vec<Nodeset>,
    queues: Vec<Queue>,
    pipelines: Vec<Pipeline>,
    secrets: Vec<Secret>,
}

impl ZuulConfigElements {
    pub fn new(ps: Vec<ZuulConfigParsedElement>) -> ZuulConfigElements {
        let mut zuul = ZuulConfigElements::default();

        for p in ps {
            match p {
                ZuulConfigParsedElement::Job(p) => zuul.jobs.push(p),
                ZuulConfigParsedElement::ProjectTemplate(p) => zuul.project_templates.push(p),
                ZuulConfigParsedElement::Project(p) => zuul.projects.push(p),
                ZuulConfigParsedElement::Nodeset(p) => zuul.nodesets.push(p),
                ZuulConfigParsedElement::Queue(p) => zuul.queues.push(p),
                ZuulConfigParsedElement::Pipeline(p) => zuul.pipelines.push(p),
                ZuulConfigParsedElement::Secret(p) => zuul.secrets.push(p),
            }
        }

        zuul
    }

    fn parse_doc(doc: &YValue, path: &Path) -> Vec<ZuulConfigParsedElement> {
        if let YValueYaml::Array(xs) = doc.value() {
            xs.iter()
                .filter_map(|x| ZuulConfigParsedElement::parse(x, path))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn parse_files(paths: &[PathBuf]) -> ZuulConfigElements {
        ZuulConfigElements::new(
            paths
                .iter()
                .flat_map(|path| {
                    let doc = load_yvalue(path);
                    match doc {
                        Ok(ys) => ys
                            .iter()
                            .flat_map(|y| Self::parse_doc(y, path))
                            .collect::<Vec<_>>(),
                        Err(err) => {
                            log::warn!("Failed to load path. path: {:#?}. err: {:#?}", path, err);
                            Vec::new()
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    define_as_ref!(jobs, Job);
    define_as_ref!(project_templates, ProjectTemplate);
    define_as_ref!(nodesets, Nodeset);
    define_as_ref!(queues, Queue);
    define_as_ref!(pipelines, Pipeline);
    define_as_ref!(secrets, Secret);

    define_into!(into_jobs, jobs, Job);
    define_into!(into_project_templates, project_templates, ProjectTemplate);
    define_into!(into_nodesets, nodesets, Nodeset);
    define_into!(into_queues, queues, Queue);
    define_into!(into_pipelines, pipelines, Pipeline);
    define_into!(into_secrets, secrets, Secret);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::golden_key_test::TestFiles;

    fn load_test_doc(input_path: &Path) -> Vec<ZuulConfigParsedElement> {
        let docs = load_yvalue(input_path).unwrap();
        let input_path = input_path.to_path_buf();

        docs.iter()
            .flat_map(|doc| ZuulConfigElements::parse_doc(doc, &input_path))
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_parse_job_0() {
        // Configure the test input information
        let ts = TestFiles::new("job_0.yaml");

        // Parse the input
        let es = load_test_doc(&ts.input_path);

        // Compare with the assert output
        ts.assert_output(&es);
    }

    #[test]
    fn test_parse_job_1() {
        // Configure the test input information
        let ts = TestFiles::new("job_1.yaml");

        // Parse the input
        let es = load_test_doc(&ts.input_path);

        // Compare with the assert output
        ts.assert_output(&es);
    }

    #[test]
    fn test_parse_nodeset_0() {
        // Configure the test input information
        let ts = TestFiles::new("nodeset_0.yaml");

        // Parse the input
        let es = load_test_doc(&ts.input_path);

        // Compare with the assert output
        ts.assert_output(&es);
    }
}
