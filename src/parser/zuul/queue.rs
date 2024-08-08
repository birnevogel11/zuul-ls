use std::path::PathBuf;
use std::rc::Rc;

use hashlink::LinkedHashMap;

use crate::parser::common::{parse_string_value, StringLoc, ZuulParse, ZuulParseError};
use crate::parser::yaml::YValue;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub struct Queue {
    name: StringLoc,
}

impl ZuulParse<Queue> for Queue {
    fn parse(
        xs: &LinkedHashMap<YValue, YValue>,
        path: &Rc<PathBuf>,
    ) -> Result<Queue, ZuulParseError> {
        let mut name = StringLoc::default();

        for (key, value) in xs {
            if let Some(key) = key.as_str() {
                if key == "name" {
                    name = parse_string_value(value, path, "name")?;
                }
            }
        }

        Ok(Queue { name })
    }
}
