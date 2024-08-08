use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::common::ZuulParse;
use crate::parser::job::Job;
use crate::parser::yaml::{load_yvalue, YValue, YValueYaml};

enum ZuulParseType {
    Job,
    ProjectTemplate,
}

impl ZuulParseType {
    pub fn determine(xs: &LinkedHashMap<YValue, YValue>) -> Option<ZuulParseType> {
        for (key, _) in xs {
            if let YValueYaml::String(key) = key.value() {
                if key == "job" {
                    return Some(ZuulParseType::Job);
                } else if key == "project_template" {
                    return Some(ZuulParseType::ProjectTemplate);
                }
            }
        }

        None
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum ZuulConfigElement {
    Job(Job),
}

impl ZuulConfigElement {
    pub fn parse(raw_config: &YValue, path: &Rc<PathBuf>) -> Option<ZuulConfigElement> {
        if let YValueYaml::Hash(xs) = raw_config.value() {
            match ZuulParseType::determine(xs) {
                Some(p) => match p {
                    ZuulParseType::Job => {
                        let job = Job::parse(xs, path).ok()?;
                        Some(ZuulConfigElement::Job(job))
                    }
                    ZuulParseType::ProjectTemplate => todo!(),
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
