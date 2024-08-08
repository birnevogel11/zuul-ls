use std::path::Path;
use std::path::PathBuf;

use crate::parser::zuul::parse_zuul;
use crate::parser::zuul::project_template::ProjectTemplate;
use crate::search::path::get_zuul_yaml_paths_cwd;
use crate::search::path::to_path;
use crate::search::report_print::print_project_templates;

pub fn list_project_templates(
    work_dir: &Path,
    config_path: Option<PathBuf>,
) -> Vec<ProjectTemplate> {
    let yaml_paths = get_zuul_yaml_paths_cwd(work_dir, config_path);
    parse_zuul(&yaml_paths).into_project_templates()
}

pub fn list_project_templates_cli(work_dir: &Path, config_path: Option<PathBuf>, is_local: bool) {
    let mut project_templates = list_project_templates(work_dir, config_path);
    if is_local {
        let sw = to_path(work_dir.to_str().unwrap());
        let sw = sw.to_str().unwrap();
        project_templates.retain(|x| x.name().path.to_str().unwrap().starts_with(sw));
    }
    print_project_templates(&project_templates);
}
