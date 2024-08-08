use std::path::PathBuf;

#[derive(Debug)]
pub struct Value {
    value: String,
    line_no: u32,
    column_begin: u32,
    column_end: u32,
    path: String,
}

impl Value {
    pub fn new(value: &str, path: String) -> Value {
        Value {
            value: String::from(value),
            line_no: 0,
            column_begin: 0,
            column_end: 0,
            path,
        }
    }
}

fn _should_visit_dir(path: &PathBuf) -> bool {
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

fn _tranversal_dirs(base_dir: PathBuf, check_dir_name: &str) -> Vec<PathBuf> {
    let mut xs = Vec::new();

    if let Ok(dir_iter) = base_dir.read_dir() {
        if base_dir.join(check_dir_name).is_dir() {
            xs.push(base_dir);
            return xs;
        }

        for entry in dir_iter.map_while(|x| x.ok()) {
            let path = entry.path();
            if _should_visit_dir(&path) {
                xs.append(&mut (_tranversal_dirs(path, check_dir_name)));
            }
        }
    }

    xs
}

pub fn list_repos(base_dir: PathBuf) -> Vec<PathBuf> {
    _tranversal_dirs(base_dir, "zuul.d")
}

fn list_roles_internal(base_dir: PathBuf) -> Vec<PathBuf> {
    _tranversal_dirs(base_dir, "tasks")
}

pub fn list_roles(repo_dirs: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut xs = Vec::new();

    let role_dir_iters = repo_dirs
        .iter()
        .map(|x| x.join("roles").read_dir())
        .map_while(|x| x.ok());

    for dir_iter in role_dir_iters {
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
