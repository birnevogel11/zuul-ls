use std::path::Path;
use std::path::PathBuf;

use crate::parser::zuul::parse_zuul;
use crate::parser::zuul::project_template::ProjectTemplate;
use crate::search::path::{get_repo_dirs, get_zuul_yaml_paths, shorten_path};

pub fn list_project_templates_from_cli(
    work_dir: &Path,
    config_path: Option<PathBuf>,
) -> Vec<ProjectTemplate> {
    let repo_dirs = get_repo_dirs(work_dir, config_path);
    let yaml_paths = get_zuul_yaml_paths(&repo_dirs);
    parse_zuul(&yaml_paths).into_project_templates()
}

pub fn list_project_templates(work_dir: &Path, config_path: Option<PathBuf>) {
    let project_templates = list_project_templates_from_cli(work_dir, config_path);

    for pt in project_templates {
        let name = pt.name();
        println!(
            "{}\t{}\t{}\t{}",
            name.value,
            shorten_path(&name.path).display(),
            name.line,
            name.col
        );
    }
}
