use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use glob::glob;
use path_absolutize::*;

use crate::config::{get_config, Config};

pub fn to_path(x: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(x).into_owned())
        .absolutize()
        .unwrap()
        .into_owned()
}

pub fn retrieve_repo_path(path: &str) -> PathBuf {
    let repo_path: String = to_path(path)
        .components()
        .take_while(|x| x.as_os_str() != "zuul.d")
        .map(|x| x.as_os_str().to_str().unwrap().to_string())
        .collect::<Vec<String>>()
        .join("/");

    PathBuf::from(&repo_path[1..repo_path.len()])
}

pub fn get_zuul_yaml_paths(repo_dirs: &[PathBuf]) -> Vec<Rc<PathBuf>> {
    repo_dirs
        .iter()
        .map(|x| list_zuul_yaml_paths(x))
        .collect::<Vec<_>>()
        .concat()
}

pub fn get_repo_dirs(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let work_dir = get_work_dir(work_dir);
    let config = get_config(&config_path);
    let base_dirs = find_tenant_base_dirs(config, &work_dir).unwrap_or(vec![work_dir]);

    base_dirs
        .into_iter()
        .map(|base_dir| traversal_dirs(base_dir, "zuul.d"))
        .collect::<Vec<Vec<PathBuf>>>()
        .concat()
}

pub fn get_role_repo_dirs(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let work_dir = get_work_dir(work_dir);
    let config = get_config(&config_path);
    let mut repo_dirs: Vec<PathBuf> = vec![PathBuf::from(&work_dir)];
    repo_dirs.append(&mut find_tenant_role_dirs(config, &work_dir).unwrap_or_default());
    repo_dirs
}

pub fn traversal_dirs(base_dir: PathBuf, check_dir_name: &str) -> Vec<PathBuf> {
    match base_dir.read_dir() {
        Ok(dir_iter) => {
            if base_dir.join(check_dir_name).is_dir() {
                vec![base_dir]
            } else {
                let mut xs = Vec::new();
                for entry in dir_iter.map_while(|x| x.ok()) {
                    let path = entry.path();
                    if should_visit_dir(&path) {
                        xs.append(&mut (traversal_dirs(path, check_dir_name)));
                    }
                }
                xs
            }
        }
        _ => vec![],
    }
}

fn should_visit_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    let file_name = path.file_name().unwrap().to_str().unwrap();
    for pat in [".", "node_modules", "__pycache__"] {
        if file_name.starts_with(pat) {
            return false;
        }
    }

    true
}

fn get_work_dir(work_dir: Option<PathBuf>) -> PathBuf {
    match work_dir {
        Some(work_dir) => to_path(work_dir.to_str().unwrap()),
        None => to_path("."),
    }
}

fn find_tenant_role_dirs(config: Option<Config>, work_dir: &Path) -> Option<Vec<PathBuf>> {
    find_tenant_dirs(config, work_dir, false)
}

fn find_tenant_base_dirs(config: Option<Config>, work_dir: &Path) -> Option<Vec<PathBuf>> {
    find_tenant_dirs(config, work_dir, true)
}

fn find_tenant_dirs(
    config: Option<Config>,
    work_dir: &Path,
    is_base: bool,
) -> Option<Vec<PathBuf>> {
    let config = config?;
    let tenant = config.find_tenant(work_dir)?;
    let tenant_config = config.tenants.get(&tenant)?;

    if is_base {
        let mut base_dirs = tenant_config.base_dirs.clone();
        base_dirs.append(&mut tenant_config.extra_base_dirs.clone());
        Some(base_dirs)
    } else {
        Some(tenant_config.extra_role_dirs.clone())
    }
}

fn list_zuul_yaml_paths(repo_dir: &Path) -> Vec<Rc<PathBuf>> {
    match glob(repo_dir.join("zuul.d/*.yaml").to_str().unwrap()) {
        Ok(xs) => xs
            .into_iter()
            .map_while(|x| x.ok())
            .map(Rc::new)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    }
}
