use std::path::Path;
use std::path::PathBuf;

use crate::search::path::get_role_repo_dirs;
use crate::search::path::traversal_dirs;

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
        for entry in dir_iter.map_while(|x| x.ok()) {
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
    let xs: Vec<Vec<(String, PathBuf)>> = repo_dirs
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
        .collect();
    let mut xs: Vec<(String, PathBuf)> = xs.concat();
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    xs
}

pub fn list_roles_cli(work_dir: Option<PathBuf>, config_path: Option<PathBuf>) {
    let repo_dirs = get_role_repo_dirs(work_dir, config_path);
    let role_dirs = list_roles(&repo_dirs);

    for (name, path) in role_dirs {
        println!("{} {}", name, path.to_str().unwrap());
    }
}
