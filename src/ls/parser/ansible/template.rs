use std::path::Path;

use crate::parser::yaml::load_yvalue_from_str;
use crate::parser::zuul::var_table::parse_var_table;
use crate::parser::zuul::var_table::VarTable;

pub fn parse_template_vars(content: &str, path: &Path) -> Option<VarTable> {
    let docs = load_yvalue_from_str(content).ok()?;
    let mut var_table = VarTable::new();
    for doc in docs {
        var_table.extend(parse_var_table(&doc, path, path.to_str().unwrap_or("")).ok()?);
    }
    Some(var_table)
}
