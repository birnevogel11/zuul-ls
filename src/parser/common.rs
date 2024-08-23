use std::path::Path;

use hashlink::LinkedHashMap;
use interner::global::{GlobalPath, GlobalString, PathPool, StringPool};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

use crate::parser::yaml::{YValue, YValueYaml};

pub static STRING_POOL: StringPool = StringPool::new();
pub static PATH_POOL: PathPool = PathPool::new();

pub fn from_string_pool(raw_str: &str) -> GlobalString {
    STRING_POOL.get(raw_str)
}

pub fn from_path_pool(raw_path: &Path) -> GlobalPath {
    PATH_POOL.get(raw_path)
}

pub trait ZuulParse<T> {
    fn parse(xs: &LinkedHashMap<YValue, YValue>, path: &Path) -> Result<T, ZuulParseError>;
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct StringLoc {
    pub value: GlobalString,
    pub path: GlobalPath,
    pub line: usize,
    pub col: usize,
}

impl Default for StringLoc {
    fn default() -> Self {
        StringLoc {
            value: from_string_pool(""),
            path: from_path_pool(Path::new("")),
            line: 0,
            col: 0,
        }
    }
}

impl From<StringLoc> for Location {
    fn from(val: StringLoc) -> Self {
        let line = val.line as u32;
        let begin_col = val.col as u32;
        let end_col = (val.col + val.value.len()) as u32;

        Location::new(
            Url::from_file_path(val.path.to_path_buf()).unwrap(),
            Range::new(Position::new(line, begin_col), Position::new(line, end_col)),
        )
    }
}

impl StringLoc {
    pub fn from(value: &YValue, path: &Path) -> StringLoc {
        StringLoc {
            value: from_string_pool(value.as_str().unwrap()),
            path: from_path_pool(path),
            line: value.line(),
            col: value.col(),
        }
    }

    pub fn from_simple(value: &str, path: &Path) -> StringLoc {
        StringLoc {
            value: from_string_pool(value),
            path: from_path_pool(path),
            line: 0,
            col: 0,
        }
    }

    pub fn clone_loc(&self, new_value: &str) -> StringLoc {
        StringLoc {
            value: from_string_pool(new_value),
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct ZuulParseError {
    msg: String,
    value: String,
    path: String,
    line: usize,
    col: usize,
}

impl ZuulParseError {
    pub fn from(msg: &str, value: &YValue, path: &Path) -> ZuulParseError {
        ZuulParseError {
            msg: msg.to_string(),
            value: format!("{:?}", value),
            path: path.to_str().unwrap().to_string(),
            line: value.line(),
            col: value.col(),
        }
    }
}

pub fn parse_string_value(
    value: &YValue,
    path: &Path,
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

pub fn parse_optional_string_value(
    value: &YValue,
    path: &Path,
    field_name: &str,
) -> Result<Option<StringLoc>, ZuulParseError> {
    match value.value() {
        YValueYaml::Null => Ok(None),
        _ => Ok(Some(parse_string_value(value, path, field_name)?)),
    }
}

pub fn parse_list_string_value(
    value: &YValue,
    path: &Path,
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
    path: &Path,
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
