use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use log::debug;

use dirs;
use path_absolutize::*;
use yaml_rust2::yaml::Yaml;
use yaml_rust2::yaml::YamlLoader;

extern crate shellexpand;

#[derive(Default, Debug, PartialEq)]
pub struct TenantConfig {
    pub name: String,
    pub base_dirs: Vec<PathBuf>,
    pub extra_base_dirs: Vec<PathBuf>,
    pub extra_role_dirs: Vec<PathBuf>,
}

impl TenantConfig {
    pub fn is_in_base_dirs(&self, path: &str) -> bool {
        let abs_path = to_path(path);
        self.base_dirs
            .iter()
            .map(|x| abs_path.starts_with(x))
            .reduce(|x, y| x | y)
            .unwrap()
    }
}

pub fn get_config(path: &Option<PathBuf>) -> Option<Config> {
    let config = match path {
        Some(path) => Config::read_config_path(path),
        None => Config::read_config(),
    }?;

    debug!("config: {:#?}", config);

    Some(Config::validate_config(config))
}

#[derive(Default, Debug, PartialEq)]
pub struct Config {
    pub default_tenant: String,
    pub tenants: HashMap<String, TenantConfig>,
}

impl Config {
    pub fn read_config() -> Option<Config> {
        Self::read_config_from_path(None)
    }

    pub fn read_config_path(path: &Path) -> Option<Config> {
        Self::read_config_from_path(Some(path.into()))
    }

    pub fn read_config_str(content: String) -> Option<Config> {
        let mut config = Config::default();
        if let Ok(raw_configs) = YamlLoader::load_from_str(&content) {
            if raw_configs.len() != 1 {
                return None;
            }
            let doc = &raw_configs[0];
            config.default_tenant = doc["default_tenant"].as_str().unwrap().to_string();
            let tenants = doc["tenant"].as_hash().unwrap();
            for t in tenants.iter() {
                let name = t.0.as_str().unwrap().to_string();
                let base_dirs = to_vec_paths(parse_str_or_list(&get_key_content(t.1, "base_dir")));
                let extra_base_dirs =
                    to_vec_paths(parse_str_or_list(&get_key_content(t.1, "extra_base_dir")));
                let mut extra_role_dirs =
                    to_vec_paths(parse_str_or_list(&get_key_content(t.1, "extra_role_dir")));
                extra_role_dirs.append(&mut make_common_roles_dir(&base_dirs));

                config.tenants.insert(
                    name.clone(),
                    TenantConfig {
                        name,
                        base_dirs,
                        extra_base_dirs,
                        extra_role_dirs,
                    },
                );
            }
        }
        Some(config)
    }

    pub fn validate_config(config: Config) -> Config {
        let default_tenant = config.default_tenant;
        let mut tenants = HashMap::new();

        for t in config.tenants {
            let name = t.0;
            let tenant = t.1;

            tenants.insert(
                name,
                TenantConfig {
                    name: tenant.name,
                    base_dirs: filter_valid_paths(tenant.base_dirs),
                    extra_base_dirs: filter_valid_paths(tenant.extra_base_dirs),
                    extra_role_dirs: filter_valid_paths(tenant.extra_role_dirs),
                },
            );
        }

        Config {
            default_tenant,
            tenants,
        }
    }

    pub fn find_tenant(&self, work_dir: &Path) -> Option<String> {
        for tenant in &self.tenants {
            let name = tenant.0;
            let tenant_config = tenant.1;

            if tenant_config.is_in_base_dirs(work_dir.to_str().unwrap()) {
                return Some(name.clone());
            }
        }
        None
    }

    fn read_config_from_path(custom_path: Option<PathBuf>) -> Option<Config> {
        match Self::read_config_file(custom_path) {
            Some(content) => Self::read_config_str(content),
            None => None,
        }
    }

    fn read_config_file(custom_path: Option<PathBuf>) -> Option<String> {
        let config_path = determine_config_path(custom_path);
        debug!("config_path: {}", config_path.display());
        fs::read_to_string(config_path).ok()
    }
}

fn determine_config_path(custom_path: Option<PathBuf>) -> PathBuf {
    match (custom_path, env::var("ZUUL_SEARCH_CONFIG_PATH")) {
        (Some(path), _) => path,
        (_, Ok(path)) => path.into(),
        (_, _) => dirs::config_dir().unwrap().join("zuul-search/config.yaml"),
    }
}

fn parse_str_or_list(raw_content: &Option<&Yaml>) -> Vec<String> {
    let mut xs = Vec::new();

    if let Some(raw_content) = raw_content {
        match (raw_content.as_str(), raw_content.as_vec()) {
            (Some(path), _) => xs.push(path.to_string()),
            (_, Some(ref mut ys)) => {
                xs.append(
                    &mut ys
                        .iter()
                        .map(|y| String::from(y.as_str().unwrap()))
                        .collect(),
                );
            }
            (None, None) => unreachable!(),
        }
    }

    xs
}

fn to_path(x: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(x).into_owned())
        .absolutize()
        .unwrap()
        .into_owned()
}

fn to_vec_paths(xs: Vec<String>) -> Vec<PathBuf> {
    xs.iter().map(|x| to_path(x.as_str())).collect()
}

fn make_common_roles_dir(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut ys = Vec::new();

    for name in ["zuul-shared", "zuul-trusted"] {
        ys.append(&mut base_dirs.iter().map(|x| x.join(name)).collect());
    }

    ys
}

fn get_key_content<'a>(raw_config: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    if let Some(raw_config) = raw_config.as_hash() {
        let search_key = Yaml::String(key.to_owned());
        return raw_config.get(&search_key);
    }
    None
}

fn filter_valid_paths(xs: Vec<PathBuf>) -> Vec<PathBuf> {
    xs.iter().filter_map(|x| fs::canonicalize(x).ok()).collect()
}

pub fn get_work_dir(work_dir: Option<PathBuf>) -> PathBuf {
    match work_dir {
        Some(work_dir) => to_path(work_dir.to_str().unwrap()),
        None => to_path("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_str() {
        let raw_str = r#"
            default_tenant: "bar"

            tenant:
              bar:
                base_dir: ~/foo/bar
                extra_base_dir:
                  - ~/foo/another
                extra_role_dir:
                  - ~/foo/another/extra_role
                  - ~/foo/zar/extra-role2
        "#;

        let tenant = TenantConfig {
            name: "bar".into(),
            base_dirs: vec![to_path("~/foo/bar")],
            extra_base_dirs: vec![to_path("~/foo/another")],
            extra_role_dirs: vec![
                to_path("~/foo/another/extra_role"),
                to_path("~/foo/zar/extra-role2"),
                to_path("~/foo/bar/zuul-shared"),
                to_path("~/foo/bar/zuul-trusted"),
            ],
        };

        let config = Config {
            default_tenant: "bar".into(),
            tenants: HashMap::from([("bar".into(), tenant)]),
        };

        assert_eq!(config, Config::read_config_str(raw_str.into()).unwrap());
    }
}
