use std::path::Path;
use std::path::PathBuf;

use log;

use crate::path::{get_role_repo_dirs, shorten_path, to_path, traversal_dirs};
use crate::safe_println;

fn get_roles_prefix_dir(repo_dir: &Path) -> String {
    let mut raw_path: String = repo_dir.to_str().unwrap().into();
    raw_path.push_str(if raw_path.ends_with('/') {
        "roles/"
    } else {
        "/roles/"
    });

    raw_path
}

fn list_roles_internal(base_dir: PathBuf) -> Vec<PathBuf> {
    traversal_dirs(base_dir, "tasks")
}

fn list_role_dir(repo_dir: &Path) -> Vec<PathBuf> {
    let mut xs = Vec::new();

    let role_dir = repo_dir.join("roles");
    if let Ok(dir_iter) = role_dir.read_dir() {
        for entry in dir_iter.filter_map(|x| x.ok()) {
            let path = entry.path();
            if path.join("tasks").is_dir() {
                xs.push(path);
            } else {
                xs.append(&mut list_roles_internal(path));
            }
        }
    }

    xs
}

pub fn list_roles(repo_dirs: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut xs = repo_dirs
        .iter()
        .map(|repo_dir| {
            let raw_path = get_roles_prefix_dir(repo_dir);
            let raw_path = raw_path.as_str();

            list_role_dir(repo_dir)
                .iter()
                .map(|x| {
                    let role_name: String = x.to_str().unwrap().into();
                    let role_name: String = role_name.strip_prefix(raw_path).unwrap().into();
                    (role_name, x.join("tasks/main.yaml"))
                })
                .collect()
        })
        .collect::<Vec<Vec<_>>>()
        .concat();
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    xs
}

pub fn list_roles_cli(work_dir: &PathBuf, config_path: Option<PathBuf>, is_local: bool) {
    let repo_dirs = get_role_repo_dirs(work_dir, config_path);

    let mut role_dirs: Vec<(String, PathBuf)> = list_roles(&repo_dirs);
    if is_local {
        let sw = to_path(work_dir.to_str().unwrap());
        let sw = sw.to_str().unwrap();
        role_dirs.retain(|(_, path)| {
            let s = to_path(path.to_str().unwrap());
            s.to_str().unwrap().starts_with(sw)
        });
    }
    let role_dirs = role_dirs;

    log::debug!("work_dir: {}", work_dir.display());
    log::debug!("role_repo_dirs: {:#?}", repo_dirs);

    for (name, path) in role_dirs {
        safe_println!("{}\t{}", name, shorten_path(&path).display());
    }
}
