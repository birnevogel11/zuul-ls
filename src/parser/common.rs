use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::yaml::YValue;

pub trait ZuulParse<T> {
    fn parse(xs: &LinkedHashMap<YValue, YValue>, path: &Rc<PathBuf>) -> Result<T, ZuulParseError>;
}

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

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct StringLoc {
    pub value: String,
    pub path: Rc<PathBuf>,
    pub line: usize,
    pub col: usize,
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

    pub fn assign_value(&self, new_value: String) -> StringLoc {
        StringLoc {
            value: new_value,
            ..self.clone()
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

pub fn parse_string_value(
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

pub fn parse_list_string_value(
    value: &YValue,
    path: &Rc<PathBuf>,
    field_name: &str,
) -> Result<Vec<StringLoc>, ZuulParseError> {
    let mut ys = Vec::new();
    match value.as_vec() {
        Some(xs) => {
            for x in xs {
                let y = parse_string_value(x, path, field_name)?;
                ys.push(y);
            }
        }
        None => {
            return Err(ZuulParseError::from(
                format!("Failed to parse the value of {}", field_name)
                    .to_string()
                    .as_str(),
                value,
                path,
            ));
        }
    }
    Ok(ys)
}

pub fn parse_string_or_list_string(
    value: &YValue,
    path: &Rc<PathBuf>,
    field_name: &str,
) -> Result<Vec<StringLoc>, ZuulParseError> {
    match (
        parse_string_value(value, path, field_name),
        parse_list_string_value(value, path, field_name),
    ) {
        (Ok(_), Ok(_)) => unreachable!(),
        (Ok(value), Err(_)) => Ok(vec![value]),
        (Err(_), Ok(value)) => Ok(value),
        (Err(_), Err(_)) => Err(ZuulParseError::from(
            format!("Failed to parse the value of {}", field_name)
                .to_string()
                .as_str(),
            value,
            path,
        )),
    }
}
