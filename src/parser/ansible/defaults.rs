use std::path::Path;

use crate::parser::var_table::{parse_var_group, VarGroup, VariableSource};
use crate::parser::yaml::load_yvalue_from_str;

pub fn parse_defaults_vars(
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
    let source = VariableSource::from_role(role_name, role_path);
    parse_var_group(doc, path, role_name, &source).ok()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::golden_key_test::TestFiles;

    use super::*;

    #[test]
    fn test_parse_defaults_vars() {
        let ts = TestFiles::new("ansible_default.yaml");
        let xs = parse_defaults_vars(
            &ts.read_input(),
            &PathBuf::from("/fake/play.yaml"),
            "fake_role",
            &PathBuf::from("/fake/roles/fake_role"),
        );

        let xs = if let Some(xs) = xs {
            let mut var_group = xs.into_iter().collect::<Vec<_>>();
            var_group.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            Some(var_group)
        } else {
            None
        };

        ts.assert_output(&xs);
    }
}
