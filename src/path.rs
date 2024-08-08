use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::debug;

use path_absolutize::*;
use walkdir::WalkDir;

extern crate dirs;

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

pub fn get_zuul_yaml_paths(repo_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let paths = repo_dirs
        .iter()
        .map(|x| list_zuul_yaml_paths(x))
        .collect::<Vec<_>>()
        .concat();
    debug!("yaml_paths: {:#?}", paths);
    paths
}

pub fn get_zuul_yaml_paths_cwd(work_dir: &Path, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let repo_dirs = get_repo_dirs(work_dir, config_path);
    get_zuul_yaml_paths(&repo_dirs)
}

pub fn get_repo_dirs(work_dir: &Path, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let config = get_config(&config_path);
    // Assume the parent dir of the work dir is the base dir when the config
    // is undefined.
    let base_dirs = find_tenant_base_dirs(config, work_dir)
        .unwrap_or(vec![PathBuf::from(work_dir.parent().unwrap_or(work_dir))]);

    let repo_dirs = base_dirs
        .into_iter()
        .map(|base_dir| traversal_dirs(base_dir, "zuul.d"))
        .collect::<Vec<Vec<PathBuf>>>()
        .concat();

    debug!("repo_dirs: {:#?}", repo_dirs);
    repo_dirs
}

pub fn get_role_repo_dirs(work_dir: &PathBuf, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let config = get_config(&config_path);
    let mut repo_dirs: Vec<PathBuf> = vec![PathBuf::from(work_dir)];
    repo_dirs.append(&mut find_tenant_role_dirs(config, work_dir).unwrap_or_default());
    repo_dirs
}

pub fn traversal_dirs(base_dir: PathBuf, check_dir_name: &str) -> Vec<PathBuf> {
    match base_dir.read_dir() {
        Ok(dir_iter) => {
            if base_dir.join(check_dir_name).is_dir() {
                vec![base_dir]
            } else {
                let mut xs = Vec::new();
                for entry in dir_iter.filter_map(|x| x.ok()) {
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
    let tenant_config = config.get_tenant(&tenant)?;

    if is_base {
        let mut base_dirs = tenant_config.base_dirs.clone();
        base_dirs.append(&mut tenant_config.extra_base_dirs.clone());
        Some(base_dirs)
    } else {
        Some(tenant_config.extra_role_dirs.clone())
    }
}

fn list_zuul_yaml_paths(repo_dir: &Path) -> Vec<PathBuf> {
    let mut zuul_config_dirs = vec![repo_dir.join("zuul.d")];
    for entry in repo_dir.read_dir().unwrap().filter_map(|x| x.ok()) {
        let path = entry.path();
        if path.is_dir() && path.to_str().unwrap().ends_with("zuul-extra.d") {
            zuul_config_dirs.push(path)
        }
    }

    zuul_config_dirs
        .into_iter()
        .map(|dir_path| {
            WalkDir::new(dir_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|x| x.file_name().to_str().unwrap().ends_with(".yaml"))
                .map(|x| x.into_path())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>()
        .concat()
}

pub fn shorten_path(path: &Path) -> PathBuf {
    let work_dir = to_path(".");
    let work_dir = work_dir.to_str().unwrap();

    let path = path.to_str().unwrap();
    let path = if path.starts_with(work_dir) {
        let mut p = ".".to_string();
        p.push_str(path.strip_prefix(work_dir).unwrap());
        p
    } else {
        path.to_string()
    };

    PathBuf::from(path)
}

pub fn filter_valid_paths(xs: Vec<PathBuf>) -> Vec<PathBuf> {
    xs.iter()
        .filter_map(|x| fs::canonicalize(to_path(x.to_str().unwrap())).ok())
        .collect()
}
