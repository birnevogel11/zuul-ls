use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use log::debug;

use dirs;
use yaml_rust2::yaml::Yaml;
use yaml_rust2::yaml::YamlLoader;

use crate::path::filter_valid_paths;
use crate::path::to_path;

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
    };
    debug!("config: {:#?}", config);

    config
}

#[derive(Default, Debug, PartialEq)]
pub struct Config {
    tenants: HashMap<String, TenantConfig>,
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
        if let Ok(docs) = YamlLoader::load_from_str(&content) {
            if docs.len() != 1 {
                return None;
            }

            let doc = &docs[0];
            let tenants = {
                let tenants = &doc["tenant"];
                if tenants.is_badvalue() {
                    return None;
                }
                tenants.as_hash()?
            };
            for (name, value) in tenants.iter() {
                let name = name.as_str()?;
                let base_dirs = get_key_content_pathbuf(value, "base_dir")?;
                let extra_base_dirs = get_key_content_pathbuf(value, "extra_base_dir")?;
                let mut extra_role_dirs = get_key_content_pathbuf(value, "extra_role_dir")?;
                extra_role_dirs.append(&mut make_common_roles_dir(&base_dirs));

                config.tenants.insert(
                    name.to_string(),
                    TenantConfig {
                        name: name.to_string(),
                        base_dirs,
                        extra_base_dirs,
                        extra_role_dirs,
                    },
                );
            }
        }

        Some(Self::validate_config(config))
    }

    pub fn validate_config(config: Config) -> Config {
        Config {
            tenants: config
                .tenants
                .into_iter()
                .map(|(name, tenant)| {
                    (
                        name,
                        TenantConfig {
                            name: tenant.name,
                            base_dirs: filter_valid_paths(tenant.base_dirs),
                            extra_base_dirs: filter_valid_paths(tenant.extra_base_dirs),
                            extra_role_dirs: filter_valid_paths(tenant.extra_role_dirs),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn find_tenant(&self, work_dir: &Path) -> Option<String> {
        self.tenants.iter().find_map(|(name, tenant_config)| {
            if tenant_config.is_in_base_dirs(work_dir.to_str().unwrap()) {
                Some(name.clone())
            } else {
                None
            }
        })
    }

    pub fn get_tenant(&self, name: &str) -> Option<&TenantConfig> {
        self.tenants.get(name)
    }

    fn read_config_from_path(custom_path: Option<PathBuf>) -> Option<Config> {
        match Self::read_config_file(custom_path) {
            Some(content) => Self::read_config_str(content),
            None => None,
        }
    }

    fn read_config_file(custom_path: Option<PathBuf>) -> Option<String> {
        let config_path = resolve_config_path(custom_path);
        debug!("config_path: {}", config_path.display());
        fs::read_to_string(config_path).ok()
    }
}

fn resolve_config_path(custom_path: Option<PathBuf>) -> PathBuf {
    match (custom_path, env::var("ZUUL_SEARCH_CONFIG_PATH")) {
        (Some(path), _) => path,
        (_, Ok(path)) => path.into(),
        (_, _) => dirs::config_dir().unwrap().join("zuul-search/config.yaml"),
    }
}

fn make_common_roles_dir(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut ys = Vec::new();

    for name in ["zuul-shared", "zuul-trusted"] {
        ys.append(&mut base_dirs.iter().map(|x| x.join(name)).collect());
    }

    ys
}

fn get_key_content_pathbuf(value: &Yaml, key: &str) -> Option<Vec<PathBuf>> {
    let raw_config = value.as_hash()?;
    let search_key = Yaml::String(key.to_string());
    let raw_content = raw_config.get(&search_key)?;

    Some(match (raw_content.as_str(), raw_content.as_vec()) {
        (Some(path), _) => vec![PathBuf::from(path)],
        (_, Some(ys)) => ys
            .iter()
            .map_while(|y| y.as_str())
            .map(PathBuf::from)
            .collect(),
        (None, None) => unreachable!(),
    })
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

    // #[test]
    // fn test_read_config_str() {
    //     let raw_str = r#"
    //         default_tenant: "bar"
    //
    //         tenant:
    //           bar:
    //             base_dir: ~/foo/bar
    //             extra_base_dir:
    //               - ~/foo/another
    //             extra_role_dir:
    //               - ~/foo/another/extra_role
    //               - ~/foo/zar/extra-role2
    //     "#;
    //
    //     let tenant = TenantConfig {
    //         name: "bar".into(),
    //         base_dirs: vec![to_path("~/foo/bar")],
    //         extra_base_dirs: vec![to_path("~/foo/another")],
    //         extra_role_dirs: vec![
    //             to_path("~/foo/another/extra_role"),
    //             to_path("~/foo/zar/extra-role2"),
    //             to_path("~/foo/bar/zuul-shared"),
    //             to_path("~/foo/bar/zuul-trusted"),
    //         ],
    //     };
    //
    //     let config = Config {
    //         tenants: HashMap::from([("bar".into(), tenant)]),
    //     };
    //
    //     assert_eq!(config, Config::read_config_str(raw_str.into()).unwrap());
    // }
}
