use std::path::Path;

use hashlink::LinkedHashMap;

use crate::parser::common::{parse_string_value, StringLoc, ZuulParse, ZuulParseError};
use crate::parser::yaml::YValue;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct Pipeline {
    name: StringLoc,
}

impl ZuulParse<Pipeline> for Pipeline {
    fn parse(xs: &LinkedHashMap<YValue, YValue>, path: &Path) -> Result<Pipeline, ZuulParseError> {
        let mut name = StringLoc::default();

        for (key, value) in xs {
            if let Some(key) = key.as_str() {
                if key == "name" {
                    name = parse_string_value(value, path, "name")?;
                }
            }
        }

        Ok(Pipeline { name })
    }
}
