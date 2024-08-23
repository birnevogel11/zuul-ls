mod group;
mod table;

use std::path::Path;

use interner::global::{GlobalPath, GlobalString};

use crate::parser::common::{from_path_pool, from_string_pool, StringLoc};

pub use group::{
    VariableGroup, VariableGroupInfo, VariableInfo, VariablePrintInfo, ARRAY_INDEX_KEY,
};
pub use table::{Variable, VariableTable};

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
            name: from_string_pool(name),
            path: from_path_pool(path),
        }
    }

    pub fn from_playbook(path: &Path) -> Self {
        Self::Playbook(from_path_pool(path))
    }
}
