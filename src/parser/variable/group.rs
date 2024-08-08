use hashlink::LinkedHashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use interner::global::{GlobalPath, GlobalString};

use crate::parser::common::{
    parse_string_value, StringLoc, ZuulParseError, PATH_POOL, STRING_POOL,
};
use crate::parser::yaml::{YValue, YValueYaml};

use super::VariableSource;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct VariableGroup(LinkedHashMap<GlobalString, VariableGroupInfo>);

impl Deref for VariableGroup {
    type Target = LinkedHashMap<GlobalString, VariableGroupInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct VariableInfo {
    pub name: StringLoc,
    pub value: String,
    pub source: VariableSource,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub struct VariableGroupInfo {
    pub variable_locs: Vec<VariableInfo>,
    pub members: VariableGroup,
}
