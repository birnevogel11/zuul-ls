use std::path::Path;
use std::path::PathBuf;

use dashmap::DashMap;

use crate::config::get_work_dir;
use crate::parser::common::StringLoc;
use crate::parser::variable::VariableGroup;
use crate::path::get_role_repo_dirs;
use crate::search::jobs::list_job_locs_by_name;
use crate::search::jobs::list_jobs;
use crate::search::roles::list_roles;
use crate::search::work_dir_vars::list_work_dir_vars_with_zuul_jobs;

use super::parser::TokenFileType;

#[derive(Clone, Debug, Default)]
pub struct ZuulSymbol {
    role_dirs: DashMap<String, PathBuf>,
    jobs: DashMap<String, Vec<StringLoc>>,

    vars: VariableGroup,
    role_docs: DashMap<String, Option<String>>,
}

impl ZuulSymbol {
    pub fn role_dirs(&self) -> &DashMap<String, PathBuf> {
        &self.role_dirs
    }

    pub fn vars(&self) -> &VariableGroup {
        &self.vars
    }

    pub fn jobs(&self) -> &DashMap<String, Vec<StringLoc>> {
        &self.jobs
    }

    pub fn role_docs(&self) -> &DashMap<String, Option<String>> {
        &self.role_docs
    }

    pub fn initialize(&self) {
        self.initialize_roles();
        self.initialize_jobs();
    }

    pub fn update(&self, path: &Path) {
        self.role_dirs.clear();
        self.vars.clear();
        self.jobs.clear();

        let file_type = TokenFileType::parse_path(path);
        if let Some(file_type) = file_type {
            match file_type {
                TokenFileType::ZuulConfig => {
                    self.initialize_jobs();
                }
                TokenFileType::AnsibleRoleDefaults
                | TokenFileType::AnsibleRoleTasks { .. }
                | TokenFileType::AnsibleRoleTemplates { .. } => {
                    self.initialize_roles();
                }
                TokenFileType::Unknown | TokenFileType::Playbooks => {}
            }
        }
    }

    fn initialize_roles(&self) {
        let work_dir = get_work_dir(None);
        let repo_dirs = get_role_repo_dirs(&work_dir, None);
        let role_dirs = list_roles(&repo_dirs);
        role_dirs.into_iter().for_each(|(name, path, doc)| {
            self.role_dirs.insert(name.clone(), path);
            self.role_docs.insert(name, doc);
        });
    }

    fn initialize_jobs(&self) {
        let work_dir = get_work_dir(None);
        let zuul_jobs = list_jobs(&work_dir, None);

        let jobs = list_job_locs_by_name(&zuul_jobs);
        jobs.into_iter().for_each(|(name, job_locs)| {
            self.jobs.insert(name, job_locs);
        });

        let vars = list_work_dir_vars_with_zuul_jobs(&zuul_jobs, &work_dir);
        vars.iter().for_each(|entry| {
            self.vars.insert(entry.key().clone(), entry.value().clone());
        });
    }
}
