use std::path::PathBuf;

use dashmap::DashMap;

use crate::config::get_work_dir;
use crate::parser::common::StringLoc;
use crate::parser::var_table::VariableInfo;
use crate::path::get_role_repo_dirs;
use crate::search::jobs::list_job_locs_by_name;
use crate::search::roles::list_roles;
use crate::search::work_dir_vars::list_work_dir_vars_group;

#[derive(Clone, Debug, Default)]
pub struct ZuulSymbol {
    role_dirs: DashMap<String, PathBuf>,
    vars: DashMap<String, Vec<VariableInfo>>,
    jobs: DashMap<String, Vec<StringLoc>>,
}

impl ZuulSymbol {
    pub fn role_dirs(&self) -> &DashMap<String, PathBuf> {
        &self.role_dirs
    }

    pub fn vars(&self) -> &DashMap<String, Vec<VariableInfo>> {
        &self.vars
    }

    pub fn jobs(&self) -> &DashMap<String, Vec<StringLoc>> {
        &self.jobs
    }

    pub fn initialize(&self) {
        let work_dir = get_work_dir(None);
        let repo_dirs = get_role_repo_dirs(&work_dir, None);
        let role_dirs: Vec<(String, PathBuf)> = list_roles(&repo_dirs);

        role_dirs.into_iter().for_each(|(name, path)| {
            self.role_dirs.insert(name, path);
        });

        let vars = list_work_dir_vars_group(&work_dir, None);
        vars.into_iter().for_each(|(name, var_info)| {
            self.vars.insert(name, var_info);
        });

        let jobs = list_job_locs_by_name(&work_dir, None);
        jobs.into_iter().for_each(|(name, job_locs)| {
            self.jobs.insert(name, job_locs);
        })
    }
}
