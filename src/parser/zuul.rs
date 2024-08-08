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
pub enum ZuulConfigElement {
    Job(Job),
    ProjectTemplate(ProjectTemplate),
    Nodeset(Nodeset),
    Queue(Queue),
    Pipeline(Pipeline),
    Secret(Secret),
}

impl ZuulConfigElement {
    pub fn parse(raw_config: &YValue, path: &Rc<PathBuf>) -> Option<ZuulConfigElement> {
        if let YValueYaml::Hash(xs) = raw_config.value() {
            match ZuulParseType::determine(xs) {
                Some(p) => match p {
                    ZuulParseType::Job => Some(ZuulConfigElement::Job(Job::parse(xs, path).ok()?)),
                    ZuulParseType::ProjectTemplate => Some(ZuulConfigElement::ProjectTemplate(
                        ProjectTemplate::parse(xs, path).ok()?,
                    )),
                    ZuulParseType::Nodeset => {
                        Some(ZuulConfigElement::Nodeset(Nodeset::parse(xs, path).ok()?))
                    }
                    ZuulParseType::Queue => {
                        Some(ZuulConfigElement::Queue(Queue::parse(xs, path).ok()?))
                    }
                    ZuulParseType::Pipeline => {
                        Some(ZuulConfigElement::Pipeline(Pipeline::parse(xs, path).ok()?))
                    }
                    ZuulParseType::Secret => {
                        Some(ZuulConfigElement::Secret(Secret::parse(xs, path).ok()?))
                    }
                },
                None => None,
            }
        } else {
            None
        }
    }
}

fn parse_doc(doc: &YValue, path: &Rc<PathBuf>) -> Vec<ZuulConfigElement> {
    if let YValueYaml::Array(xs) = doc.value() {
        xs.iter()
            .map_while(|x| ZuulConfigElement::parse(x, path))
            .collect()
    } else {
        vec![]
    }
}

pub fn job_parser_study(path: &Path) {
    let path = crate::search::path::to_path(path.to_str().unwrap());
    let docs = load_yvalue(&path).unwrap();
    let path = Rc::new(path);

    let elements: Vec<Vec<_>> = docs.iter().map(|doc| parse_doc(doc, &path)).collect();
    let ys = elements.concat();
    println!("{:?}", ys);
}
