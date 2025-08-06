use std::path::Path;
use std::path::PathBuf;

use dashmap::DashMap;

use crate::ls::parser::AnsibleRolePath;
use crate::parser::common::StringLoc;
use crate::parser::variable::VariableGroup;
use crate::parser::zuul::ZuulConfig;
use crate::path::list_role_repo_dirs;
use crate::path::list_zuul_yaml_paths;
use crate::path::resolve_work_dir;
use crate::search::jobs::list_job_locs_by_name;
use crate::search::jobs::ZuulJobs;
use crate::search::roles::list_roles;
use crate::search::work_dir_vars::list_work_dir_vars_with_zuul_jobs;

use super::parser::TokenFileType;

#[derive(Clone, Debug, Default)]
pub struct ZuulSymbol {
    role_dirs: DashMap<String, PathBuf>,
    role_docs: DashMap<String, Option<String>>,

    jobs: DashMap<String, Vec<StringLoc>>,
    vars: VariableGroup,
    project_templates: DashMap<String, StringLoc>,
    project_template_docs: DashMap<String, String>,
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

    pub fn project_templates(&self) -> &DashMap<String, StringLoc> {
        &self.project_templates
    }

    pub fn project_template_docs(&self) -> &DashMap<String, String> {
        &self.project_template_docs
    }

    pub fn initialize(&self) {
        self.initialize_roles();
        self.initialize_jobs();
    }

    pub fn update(&self, path: &Path) {
        let file_type = TokenFileType::parse_path(path);
        if let Some(file_type) = file_type {
            match file_type {
                TokenFileType::ZuulConfig => {
                    self.vars.clear();
                    self.jobs.clear();

                    self.initialize_jobs();
                }
                TokenFileType::AnsibleRoleDefaults
                | TokenFileType::AnsibleRoleTasks { .. }
                | TokenFileType::AnsibleRoleTemplates { .. } => {
                    self.role_dirs.clear();

                    self.initialize_roles();
                }
                TokenFileType::Unknown | TokenFileType::Playbooks => {}
            }
        }
    }

    fn initialize_roles(&self) {
        let work_dir = resolve_work_dir(None);
        let repo_dirs = list_role_repo_dirs(&work_dir, None);
        let role_dirs = list_roles(&repo_dirs);
        role_dirs.into_iter().for_each(|(name, path, doc)| {
            self.role_dirs.insert(name.clone(), path);
            self.role_docs.insert(name, doc);
        });
    }

    fn initialize_jobs(&self) {
        let work_dir = resolve_work_dir(None);
        let yaml_paths = list_zuul_yaml_paths(&work_dir, None);
        let zuul_config_elements = ZuulConfig::parse_files(&yaml_paths);

        let zuul_jobs = ZuulJobs::from_parsed_jobs(zuul_config_elements.jobs().clone());
        let jobs = list_job_locs_by_name(&zuul_jobs);
        jobs.into_iter().for_each(|(name, job_locs)| {
            self.jobs.insert(name, job_locs);
        });

        let vars = list_work_dir_vars_with_zuul_jobs(&zuul_jobs, &work_dir);
        vars.iter().for_each(|entry| {
            self.vars.insert(entry.key().clone(), entry.value().clone());
        });

        let project_templates = zuul_config_elements.project_templates();
        project_templates.iter().for_each(|pt| {
            let name = pt.name();
            let description = pt.description();

            self.project_templates
                .insert(name.value.to_string(), name.clone());
            self.project_template_docs
                .insert(name.value.to_string(), description.value.to_string());
        });
    }

    pub fn get_role_path(&self, role_name: &str) -> Option<AnsibleRolePath> {
        let entry = self.role_dirs.get(role_name)?;
        let path = entry.value();
        let role_dir = path.ancestors().find(|path| {
            if let Some(path_str) = path.to_str() {
                path_str.ends_with(role_name)
            } else {
                false
            }
        })?;

        Some(AnsibleRolePath::new(role_dir))
    }
}
