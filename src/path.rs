use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log;

use path_absolutize::*;
use walkdir::WalkDir;

extern crate dirs;

use crate::config::get_config;
use crate::config::ParseConfigError;
use crate::config::{get_config_simple, Config};

pub fn to_path(x: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(x).into_owned())
        .absolutize()
        .unwrap()
        .into_owned()
}

pub fn filter_valid_paths(xs: Vec<PathBuf>) -> Vec<PathBuf> {
    xs.iter()
        .filter_map(|x| fs::canonicalize(to_path(x.to_str().unwrap())).ok())
        .collect()
}

pub fn resolve_work_dir(work_dir: Option<PathBuf>) -> PathBuf {
    to_path(work_dir.as_ref().map_or(".", |p| p.to_str().unwrap()))
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

pub fn retrieve_repo_path(path: &Path) -> Option<PathBuf> {
    path.ancestors().find_map(|x| {
        let mut base_path = x.to_path_buf();
        base_path.push("zuul.d");
        base_path.is_dir().then_some(x.to_path_buf())
    })
}

pub fn list_zuul_yaml_paths_simple(work_dir: &Path, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let config = get_config_simple(&config_path);
    let repo_dirs = list_repo_dirs(work_dir, config);

    let paths = repo_dirs
        .iter()
        .flat_map(|x| list_repo_zuul_yaml_paths(x))
        .collect::<Vec<_>>();

    log::debug!("yaml_paths: {:#?}", paths);
    paths
}

pub fn list_zuul_yaml_paths(
    work_dir: &Path,
    config_path: Option<PathBuf>,
) -> Result<Vec<PathBuf>, ParseConfigError> {
    let config = get_config(&config_path)?;
    let repo_dirs = list_repo_dirs(work_dir, Some(config));

    let paths = repo_dirs
        .iter()
        .flat_map(|x| list_repo_zuul_yaml_paths(x))
        .collect::<Vec<_>>();

    log::debug!("yaml_paths: {:#?}", paths);

    Ok(paths)
}

fn list_repo_dirs(work_dir: &Path, config: Option<Config>) -> Vec<PathBuf> {
    // Assume the parent dir of the work dir is the base dir when the config
    // is undefined.
    let base_dirs = find_tenant_base_dirs(config, work_dir)
        .unwrap_or(vec![PathBuf::from(work_dir.parent().unwrap_or(work_dir))]);

    let repo_dirs = base_dirs
        .into_iter()
        .flat_map(|base_dir| traversal_dirs(base_dir, "zuul.d"))
        .collect::<Vec<_>>();

    log::debug!("repo_dirs: {:#?}", repo_dirs);
    repo_dirs
}

pub fn list_role_repo_dirs(work_dir: &PathBuf, config_path: Option<PathBuf>) -> Vec<PathBuf> {
    let config = get_config_simple(&config_path);
    let mut repo_dirs: Vec<PathBuf> = vec![PathBuf::from(work_dir)];
    repo_dirs.append(&mut find_tenant_role_dirs(config, work_dir).unwrap_or_default());
    repo_dirs
}

fn traversal_dirs(base_dir: PathBuf, check_dir_name: &str) -> Vec<PathBuf> {
    if !base_dir.is_dir() {
        Vec::default()
    } else if base_dir.join(check_dir_name).is_dir() {
        vec![base_dir]
    } else {
        let mut xs = Vec::new();
        base_dir
            .read_dir()
            .unwrap()
            .filter_map(|x| x.ok())
            .for_each(|entry| {
                let path = entry.path();
                if should_visit_dir(&path) {
                    xs.append(&mut (traversal_dirs(path, check_dir_name)));
                }
            });
        xs
    }
}

fn should_visit_dir(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_str().unwrap();

    path.is_dir()
        && !name.starts_with(".")
        && !["node_modules", "__pycache__"]
            .into_iter()
            .all(|x| x == name)
}

fn find_tenant_role_dirs(config: Option<Config>, work_dir: &Path) -> Option<Vec<PathBuf>> {
    find_dirs(config, work_dir, false)
}

fn find_tenant_base_dirs(config: Option<Config>, work_dir: &Path) -> Option<Vec<PathBuf>> {
    find_dirs(config, work_dir, true)
}

/// List all directories for zuul configs(is_base == true) or ansible roles(is_base == false)
fn find_dirs(config: Option<Config>, work_dir: &Path, is_base: bool) -> Option<Vec<PathBuf>> {
    let config = config?;
    let tenant = config.find_tenant(work_dir)?;
    let tenant_config = config.get_tenant(&tenant)?;

    Some(if is_base {
        let mut base_dirs = tenant_config.base_dirs.clone();
        base_dirs.append(&mut tenant_config.extra_base_dirs.clone());
        base_dirs
    } else {
        tenant_config.extra_role_dirs.clone()
    })
}

fn list_repo_zuul_yaml_paths(repo_dir: &Path) -> Vec<PathBuf> {
    repo_dir
        .read_dir()
        .unwrap()
        .filter_map(|x| {
            let entry = x.ok()?;
            let path = entry.path();
            let name = path.to_str().unwrap();
            let is_zuul_dir =
                path.is_dir() && (name.ends_with("zuul.d") || name.ends_with("zuul-extra.d"));
            is_zuul_dir.then_some(path)
        })
        .flat_map(|dir_path| {
            WalkDir::new(dir_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|x| x.file_name().to_str().unwrap().ends_with(".yaml"))
                .map(|x| x.into_path())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
