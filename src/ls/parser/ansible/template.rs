use std::path::Path;

use crate::parser::yaml::load_yvalue_from_str;
use crate::parser::zuul::var_table::{
    collect_variables, group_variables, parse_var_table, VarGroup, VariableSource,
};

pub fn parse_template_vars(
    content: &str,
    path: &Path,
    role_name: &str,
    role_path: &Path,
) -> Option<VarGroup> {
    let docs = load_yvalue_from_str(content).ok()?;
    if docs.len() != 1 {
        return None;
    }
    let doc = &docs[0];

    let var_table = parse_var_table(doc, path, path.to_str().unwrap_or("")).ok()?;

    let source = VariableSource::from_role(role_name, role_path);
    Some(group_variables(
        VarGroup::new(),
        collect_variables("", &var_table, &source),
    ))
}
