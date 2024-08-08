use std::path::{Path, PathBuf};

use crate::path::to_path;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum LSFileType {
    ZuulConfig,
    Playbooks,
    AnsibleRoleTasks,
    AnsibleRoleDefaults,
    AnsibleRoleTemplates,
}

fn retrieve_repo_path(path: &Path) -> Option<PathBuf> {
    path.ancestors().find_map(|x| {
        let mut base_path = PathBuf::from(x);
        base_path.push("zuul.d");
        if base_path.is_dir() {
            Some(PathBuf::from(x))
        } else {
            None
        }
    })
}

pub fn resolve_ls_file_type(path: &Path) -> Option<LSFileType> {
    let path = to_path(path.to_str().unwrap());
    let repo_path = retrieve_repo_path(&path)?;

    let ls_file_type = [
        ("zuul.d", LSFileType::ZuulConfig),
        ("playbooks", LSFileType::Playbooks),
    ]
    .into_iter()
    .find_map(|(name, ls_path_type)| {
        let mut base_path = PathBuf::from(&repo_path);
        base_path.push(name);
        if path.starts_with(&base_path) {
            Some(ls_path_type)
        } else {
            None
        }
    });

    if ls_file_type.is_some() {
        ls_file_type
    } else {
        let mut base_path = PathBuf::from(&repo_path);
        base_path.push("roles");
        if path.starts_with(base_path) {
            path.components().find_map(|p| {
                let p = p.as_os_str().to_str().unwrap();

                [
                    ("tasks", LSFileType::AnsibleRoleTasks),
                    ("defaults", LSFileType::AnsibleRoleDefaults),
                    ("templates", LSFileType::AnsibleRoleTemplates),
                ]
                .into_iter()
                .find_map(
                    |(name, ls_file_type)| {
                        if p == name {
                            Some(ls_file_type)
                        } else {
                            None
                        }
                    },
                )
            })
        } else {
            None
        }
    }
}
