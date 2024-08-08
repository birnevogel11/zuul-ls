use std::path::Path;
use std::path::PathBuf;

use crate::config::{get_config, Config};
use crate::search::path::{list_role_dir, to_path};

fn get_work_dir(work_dir: Option<PathBuf>) -> PathBuf {
    match work_dir {
        Some(work_dir) => work_dir,
        None => to_path("."),
    }
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

pub fn list_dir_roles(repo_dirs: &[PathBuf]) -> Vec<(String, PathBuf)> {
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

fn find_tenant_work_dir_impl(config: Option<Config>, work_dir: &Path) -> Option<Vec<PathBuf>> {
    let config = config?;
    let tenant = config.find_tenant(work_dir)?;
    let tenant_config = config.tenants.get(&tenant)?;
    Some(tenant_config.extra_role_dirs.clone())
}

fn find_repo_dirs(config: Option<Config>, work_dir: &Path) -> Vec<PathBuf> {
    let mut repo_dirs: Vec<PathBuf> = vec![PathBuf::from(work_dir)];
    repo_dirs.append(&mut find_tenant_work_dir_impl(config, work_dir).unwrap_or_default());
    repo_dirs
}

pub fn list_roles(
    work_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Vec<(String, PathBuf)> {
    let work_dir = get_work_dir(work_dir);
    let config = get_config(&config_path);
    let repo_dirs = find_repo_dirs(config, &work_dir);

    list_dir_roles(&repo_dirs)
}

pub fn list_roles_cli(
    _search_key: String,
    work_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
) {
    let role_dirs = list_roles(work_dir, config_path);
    println!("{:?}", role_dirs);
}
