use std::path::PathBuf;

use path_absolutize::*;

use crate::config::get_config;
use crate::config::Config;

fn to_path(x: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(x).into_owned())
        .absolutize()
        .unwrap()
        .into_owned()
}

fn get_work_dir(work_dir: Option<PathBuf>) -> PathBuf {
    match work_dir {
        Some(work_dir) => work_dir,
        None => to_path("."),
    }
}

fn find_tenant(config: &Option<Config>, work_dir: &PathBuf) -> Option<String> {
    if let Some(config) = config {
        for tenant in &config.tenants {
            let name = tenant.0;
            let tenant_config = tenant.1;

            if tenant_config.is_in_base_dirs(work_dir.to_str().unwrap()) {
                return Some(name.clone());
            }
        }
    }
    None
}

pub fn search_roles(search_key: String, work_dir: Option<PathBuf>, config_path: Option<PathBuf>) {
    let work_dir = get_work_dir(work_dir);

    let config = get_config(&config_path);
    println!("{:?}", work_dir);
    let tenant = find_tenant(&config, &work_dir);
    let repo_dirs = match tenant {
        Some(tenant) => {
            let config = config.unwrap();
            let tenant = config.tenants.get(&tenant).unwrap();

            let mut xs = vec![work_dir.clone()];
            xs.append(&mut tenant.extra_role_dirs.clone());

            xs
        }
        None => {
            vec![work_dir]
        }
    };

    for repo_dir in repo_dirs {
        println!("{:?}", crate::repo::list_roles(&vec![repo_dir]));
    }
}
