use std::path::Path;

use hashlink::LinkedHashMap;

use crate::parser::common::{parse_string_value, StringLoc, ZuulParse, ZuulParseError};
use crate::parser::yaml::YValue;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct ProjectTemplate {
    name: StringLoc,
}

impl ProjectTemplate {
    pub fn name(&self) -> &StringLoc {
        &self.name
    }
}

impl ZuulParse<ProjectTemplate> for ProjectTemplate {
    fn parse(
        xs: &LinkedHashMap<YValue, YValue>,
        path: &Path,
    ) -> Result<ProjectTemplate, ZuulParseError> {
        let mut name: Option<StringLoc> = None;

        for (key, value) in xs {
            if let Some(key) = key.as_str() {
                if key == "name" {
                    name = Some(parse_string_value(value, path, "name")?);
                }
            }
        }

        Ok(ProjectTemplate {
            name: name.unwrap_or_default(),
        })
    }
}
