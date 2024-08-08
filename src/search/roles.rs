use std::path::Path;
use std::path::PathBuf;

use log;

use crate::path::{get_role_repo_dirs, shorten_path, to_path};
use crate::safe_println;

fn get_role_doc(base_dir: &str, role_name: &str) -> Option<String> {
    let role_dir = PathBuf::from(base_dir).join(role_name);
    ["README.rst", "README.md"]
        .into_iter()
        .map(|name| role_dir.clone().join(name))
        .find_map(|path| {
            path.is_file()
                .then_some(std::fs::read_to_string(path).ok())
                .flatten()
        })
}

fn get_roles_prefix_dir(repo_dir: &Path) -> String {
    let mut raw_path: String = repo_dir.to_str().unwrap().into();
    raw_path.push_str(if raw_path.ends_with('/') {
        "roles/"
    } else {
        "/roles/"
    });

    raw_path
}

fn is_role(path: &Path) -> Option<PathBuf> {
    let path = path.to_path_buf();

    for check_dir_name in ["tasks", "meta"] {
        for check_filename in ["main.yaml", "main.yml"] {
            let check_path = path.join(check_dir_name).join(check_filename);
            if check_path.is_file() {
                return Some(check_path);
            }
        }
    }

    None
}

fn list_role_dir(role_dir: &Path) -> Vec<PathBuf> {
    let mut xs = Vec::new();

    if let Ok(dir_iter) = role_dir.read_dir() {
        for entry in dir_iter.filter_map(|x| x.ok()) {
            let path = entry.path();
            if path.is_dir() {
                match is_role(&path) {
                    Some(role_path) => {
                        xs.push(role_path);
                    }
                    None => {
                        xs.append(&mut list_role_dir(&path));
                    }
                }
            }
        }
    }

    xs
}

fn list_role_dir_from_repo_dir(repo_dir: &Path) -> Vec<PathBuf> {
    list_role_dir(&repo_dir.join("roles"))
}

pub fn list_roles(repo_dirs: &[PathBuf]) -> Vec<(String, PathBuf, Option<String>)> {
    let mut xs: Vec<(String, PathBuf, Option<String>)> = repo_dirs
        .iter()
        .flat_map(|repo_dir| {
            let raw_path = get_roles_prefix_dir(repo_dir);
            let raw_path = &raw_path;

            list_role_dir_from_repo_dir(repo_dir)
                .into_iter()
                .map(|path| {
                    let role_name = path
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .trim_start_matches(raw_path)
                        .trim_end_matches("/tasks")
                        .trim_end_matches("/meta")
                        .to_string();
                    let role_doc = get_role_doc(raw_path, &role_name);
                    (role_name, path, role_doc)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    xs
}

pub fn list_roles_cli(work_dir: &PathBuf, config_path: Option<PathBuf>, is_local: bool) {
    let repo_dirs = get_role_repo_dirs(work_dir, config_path);

    let mut role_dirs: Vec<(String, PathBuf, Option<String>)> = list_roles(&repo_dirs);
    if is_local {
        let sw = to_path(work_dir.to_str().unwrap());
        let sw = sw.to_str().unwrap();
        role_dirs.retain(|(_, path, _)| {
            let s = to_path(path.to_str().unwrap());
            s.to_str().unwrap().starts_with(sw)
        });
    }
    let role_dirs = role_dirs;

    log::debug!("work_dir: {}", work_dir.display());
    log::debug!("role_repo_dirs: {:#?}", repo_dirs);

    for (name, path, _) in role_dirs {
        safe_println!("{}\t{}", name, shorten_path(&path).display());
    }
}
