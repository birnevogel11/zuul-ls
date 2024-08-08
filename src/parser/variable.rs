mod group;
mod table;

use hashlink::LinkedHashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use interner::global::{GlobalPath, GlobalString};

use crate::parser::common::{
    parse_string_value, StringLoc, ZuulParseError, PATH_POOL, STRING_POOL,
};
use crate::parser::yaml::{YValue, YValueYaml};

pub use group::VariableGroup;
pub use table::Variable;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash, Default)]
pub enum VariableSource {
    #[default]
    Unknown,
    Job(StringLoc),
    Role {
        name: GlobalString,
        path: GlobalPath,
    },
    Playbook(GlobalPath),
}

impl VariableSource {
    pub fn from_role(name: &str, path: &Path) -> Self {
        Self::Role {
            name: STRING_POOL.get(name),
            path: PATH_POOL.get(path),
        }
    }

    pub fn from_playbook(path: &Path) -> Self {
        Self::Playbook(PATH_POOL.get(path))
    }
}
