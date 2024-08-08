use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::yaml::{load_yvalue, YValue, YValueYaml};

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct ZuulParseError {
    msg: String,
    value: String,
    path: String,
    line: usize,
    col: usize,
}

impl ZuulParseError {
    pub fn from(msg: &str, value: &YValue, path: &Rc<PathBuf>) -> ZuulParseError {
        ZuulParseError {
            msg: msg.to_string(),
            value: format!("{:?}", value),
            path: path.to_str().unwrap().to_string(),
            line: value.line(),
            col: value.col(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
enum ZuulParseType {
    Job,
}

impl ZuulParseType {
    pub fn determine(xs: &LinkedHashMap<YValue, YValue>) -> Option<ZuulParseType> {
        for (key, _) in xs {
            if let YValueYaml::String(key) = key.value() {
                if key == "job" {
                    return Some(ZuulParseType::Job);
                }
            }
        }

        None
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct StringLoc {
    value: String,
    path: Rc<PathBuf>,
    line: usize,
    col: usize,
}

impl StringLoc {
    pub fn from(value: &YValue, path: &Rc<PathBuf>) -> StringLoc {
        StringLoc {
            value: value.as_str().unwrap().to_string(),
            path: path.clone(),
            line: value.line(),
            col: value.col(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct Job {
    job_name: StringLoc,
    parent: Option<StringLoc>,
}

fn parse_string_value(
    value: &YValue,
    path: &Rc<PathBuf>,
    field_name: &str,
) -> Result<StringLoc, ZuulParseError> {
    match value.as_str() {
        Some(_) => Ok(StringLoc::from(value, path)),
        None => Err(ZuulParseError::from(
            format!("Failed to parse the value of {}", field_name)
                .to_string()
                .as_str(),
            value,
            path,
        )),
    }
}

impl Job {
    fn parse(
        xs: &LinkedHashMap<YValue, YValue>,
        path: &Rc<PathBuf>,
    ) -> Result<Job, ZuulParseError> {
        let mut job_name = StringLoc::default();
        let mut parent: Option<StringLoc> = None;

        for (key, value) in xs {
            match key.as_str() {
                Some(key) => match key.as_str() {
                    "job" => {
                        job_name = parse_string_value(value, path, "job").unwrap();
                    }
                    "parent" => {
                        parent = parse_string_value(value, path, "parent").ok();
                    }
                    _ => {}
                },
                None => {
                    return Err(ZuulParseError::from("Failed to parse key", key, path));
                }
            }
        }
        println!("{:?}", job_name);
        println!("{:?}", parent);
        Ok(Job { job_name, parent })
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
