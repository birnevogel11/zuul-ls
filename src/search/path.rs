use path_absolutize::*;
use std::path::Path;
use std::path::PathBuf;

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

pub fn list_role_dir(repo_dir: &Path) -> Vec<PathBuf> {
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

pub fn traversal_dirs(base_dir: PathBuf, check_dir_name: &str) -> Vec<PathBuf> {
    let mut xs = Vec::new();

    if let Ok(dir_iter) = base_dir.read_dir() {
        if base_dir.join(check_dir_name).is_dir() {
            xs.push(base_dir);
            return xs;
        }

        for entry in dir_iter.map_while(|x| x.ok()) {
            let path = entry.path();
            if should_visit_dir(&path) {
                xs.append(&mut (traversal_dirs(path, check_dir_name)));
            }
        }
    }

    xs
}

fn list_roles_internal(base_dir: PathBuf) -> Vec<PathBuf> {
    traversal_dirs(base_dir, "tasks")
}

fn should_visit_dir(path: &PathBuf) -> bool {
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
