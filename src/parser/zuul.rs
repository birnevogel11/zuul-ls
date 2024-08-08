pub mod job;
pub mod nodeset;
pub mod pipeline;
pub mod project_template;
pub mod queue;
pub mod secret;

use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

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
enum ZuulParseType {
    Job,
    ProjectTemplate,
    Nodeset,
    Queue,
    Pipeline,
    Secret,
}

static ZUUL_PARSE_KEYWORDS: phf::Map<&'static str, ZuulParseType> = phf_map! {
    "job" => ZuulParseType::Job,
    "project-template" => ZuulParseType::ProjectTemplate,
    "nodeset" => ZuulParseType::Nodeset,
    "queue" => ZuulParseType::Queue,
    "pipeline" => ZuulParseType::Pipeline,
    "secret" => ZuulParseType::Secret,
};

impl ZuulParseType {
    pub fn determine(xs: &LinkedHashMap<YValue, YValue>) -> Option<ZuulParseType> {
        for (key, _) in xs {
            if let Some(key) = key.as_str() {
                return ZUUL_PARSE_KEYWORDS.get(key).cloned();
            }
        }

        None
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
enum ZuulConfigParsedElement {
    Job(Job),
    ProjectTemplate(ProjectTemplate),
    Nodeset(Nodeset),
    Queue(Queue),
    Pipeline(Pipeline),
    Secret(Secret),
}

impl ZuulConfigParsedElement {
    fn get_config_value(
        xs: &LinkedHashMap<YValue, YValue>,
    ) -> Option<&LinkedHashMap<YValue, YValue>> {
        if let Some((_, value)) = xs.into_iter().next() {
            value.as_hash()
        } else {
            None
        }
    }
    pub fn parse(raw_config: &YValue, path: &Rc<PathBuf>) -> Option<ZuulConfigParsedElement> {
        if let YValueYaml::Hash(xs) = raw_config.value() {
            let values = ZuulConfigParsedElement::get_config_value(xs)?;

            match ZuulParseType::determine(xs) {
                Some(p) => match p {
                    ZuulParseType::Job => {
                        Some(ZuulConfigParsedElement::Job(Job::parse(values, path).ok()?))
                    }
                    ZuulParseType::ProjectTemplate => {
                        Some(ZuulConfigParsedElement::ProjectTemplate(
                            ProjectTemplate::parse(values, path).ok()?,
                        ))
                    }
                    ZuulParseType::Nodeset => Some(ZuulConfigParsedElement::Nodeset(
                        Nodeset::parse(values, path).ok()?,
                    )),
                    ZuulParseType::Queue => Some(ZuulConfigParsedElement::Queue(
                        Queue::parse(values, path).ok()?,
                    )),
                    ZuulParseType::Pipeline => Some(ZuulConfigParsedElement::Pipeline(
                        Pipeline::parse(values, path).ok()?,
                    )),
                    ZuulParseType::Secret => Some(ZuulConfigParsedElement::Secret(
                        Secret::parse(values, path).ok()?,
                    )),
                },
                None => None,
            }
        } else {
            None
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct ZuulConfigElement {
    jobs: Vec<Job>,
    project_templates: Vec<ProjectTemplate>,
    nodesets: Vec<Nodeset>,
    queues: Vec<Queue>,
    pipelines: Vec<Pipeline>,
    secrets: Vec<Secret>,
}

impl ZuulConfigElement {
    pub fn new(ps: Vec<ZuulConfigParsedElement>) -> ZuulConfigElement {
        let mut zuul = ZuulConfigElement::default();

        for p in ps {
            match p {
                ZuulConfigParsedElement::Job(p) => zuul.jobs.push(p),
                ZuulConfigParsedElement::ProjectTemplate(p) => zuul.project_templates.push(p),
                ZuulConfigParsedElement::Nodeset(p) => zuul.nodesets.push(p),
                ZuulConfigParsedElement::Queue(p) => zuul.queues.push(p),
                ZuulConfigParsedElement::Pipeline(p) => zuul.pipelines.push(p),
                ZuulConfigParsedElement::Secret(p) => zuul.secrets.push(p),
            }
        }

        zuul
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

fn parse_doc(doc: &YValue, path: &Rc<PathBuf>) -> Vec<ZuulConfigParsedElement> {
    if let YValueYaml::Array(xs) = doc.value() {
        xs.iter()
            .map_while(|x| ZuulConfigParsedElement::parse(x, path))
            .collect()
    } else {
        vec![]
    }
}

pub fn parse_zuul(paths: &[Rc<PathBuf>]) -> ZuulConfigElement {
    ZuulConfigElement::new(
        paths
            .iter()
            .map(|path| {
                let doc = load_yvalue(path);
                match doc {
                    Ok(ys) => ys
                        .iter()
                        .map(|y| parse_doc(y, path))
                        .collect::<Vec<_>>()
                        .concat(),
                    _ => Vec::new(),
                }
            })
            .collect::<Vec<_>>()
            .concat(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_key_test::TestFiles;

    fn load_test_doc(input_path: &Path) -> Vec<ZuulConfigParsedElement> {
        let docs = load_yvalue(input_path).unwrap();
        let input_path = Rc::new(input_path.to_path_buf());

        docs.iter()
            .map(|doc| parse_doc(doc, &input_path))
            .collect::<Vec<_>>()
            .concat()
    }

    #[test]
    fn test_parse_job() {
        // Configure the test input information
        let ts = TestFiles::new("test.yaml");

        // Parse the input
        let es = load_test_doc(&ts.input_path);

        // Compare with the assert output
        ts.assert_output(&es);
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
    fn test_parse_nodeset_0() {
        // Configure the test input information
        let ts = TestFiles::new("nodeset_0.yaml");

        // Parse the input
        let es = load_test_doc(&ts.input_path);

        // Compare with the assert output
        ts.assert_output(&es);
    }
}
