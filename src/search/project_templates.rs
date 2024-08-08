use std::path::Path;
use std::path::PathBuf;

use crate::parser::zuul::parse_zuul;
use crate::parser::zuul::project_template::ProjectTemplate;
use crate::search::path::get_zuul_yaml_paths_cwd;
use crate::search::report_print::print_project_templates;

pub fn list_project_templates_from_cli(
    work_dir: &Path,
    config_path: Option<PathBuf>,
) -> Vec<ProjectTemplate> {
    let yaml_paths = get_zuul_yaml_paths_cwd(work_dir, config_path);
    parse_zuul(&yaml_paths).into_project_templates()
}

pub fn list_project_templates(work_dir: &Path, config_path: Option<PathBuf>) {
    let project_templates = list_project_templates_from_cli(work_dir, config_path);
    print_project_templates(&project_templates);
}
