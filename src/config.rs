use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use log;

use dirs;
use yaml_rust2::yaml::{Yaml, YamlLoader};
use yaml_rust2::ScanError;

use crate::path::filter_valid_paths;
use crate::path::to_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormatError {
    NotSingleDocument,
    TenantsNotExist,
    TenantsNotDict,
    NameNotString(Yaml),
    ParseFieldError { tenant: String, key: String },
}

#[derive(Debug)]
pub enum ParseConfigError {
    FileError(std::io::Error),
    YamlParseError(ScanError),
    ConfigFormatError(ConfigFormatError),
}

impl ParseConfigError {
    fn format_error<T>(e: ConfigFormatError) -> Result<T, Self> {
        Err(Self::ConfigFormatError(e))
    }
}

impl From<std::io::Error> for ParseConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::FileError(e)
    }
}

impl From<ScanError> for ParseConfigError {
    fn from(e: ScanError) -> Self {
        Self::YamlParseError(e)
    }
}

impl From<ConfigFormatError> for ParseConfigError {
    fn from(e: ConfigFormatError) -> Self {
        Self::ConfigFormatError(e)
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct TenantConfig {
    pub name: String,
    pub base_dirs: Vec<PathBuf>,
    pub extra_base_dirs: Vec<PathBuf>,
    pub extra_role_dirs: Vec<PathBuf>,
}

impl TenantConfig {
    pub fn is_tenant(&self, path: &Path) -> bool {
        let path = to_path(path.to_str().unwrap());
        self.base_dirs.iter().any(|x| path.starts_with(x))
    }
}

pub fn get_config_simple(path: &Option<PathBuf>) -> Option<Config> {
    let config = get_config(path);
    if config.is_err() {
        log::warn!(
            "Failed to read config. err: {:#?}. Skip the error and continue",
            config
        );
    }
    config.ok()
}

pub fn get_config(path: &Option<PathBuf>) -> Result<Config, ParseConfigError> {
    let config_path = resolve_config_path(path);
    Ok(if !config_path.is_file() {
        log::info!("Not existed config file. path: {:#?}", config_path);
        Config::default()
    } else {
        let config = Config::parse_config_from_path(&config_path)?;
        log::debug!("original config: {:#?}", config);

        let config = Config::filter_invalid_paths(config);
        log::debug!("filtered config: {:#?}", config);

        config
    })
}

#[derive(Default, Debug, PartialEq)]
pub struct Config {
    tenants: HashMap<String, TenantConfig>,
}

impl Config {
    pub fn parse_config_from_path(config_path: &Path) -> Result<Config, ParseConfigError> {
        Self::parse_config_str(fs::read_to_string(config_path)?)
    }

    pub fn parse_config_str(content: String) -> Result<Config, ParseConfigError> {
        let mut config = Config::default();
        let docs = YamlLoader::load_from_str(&content)?;
        if docs.len() != 1 {
            return ParseConfigError::format_error(ConfigFormatError::NotSingleDocument);
        }
        let doc = &docs[0];
        let tenants = {
            let tenants = &doc["tenant"];
            if tenants.is_badvalue() {
                return ParseConfigError::format_error(ConfigFormatError::TenantsNotExist);
            }

            tenants
                .as_hash()
                .ok_or_else(|| ConfigFormatError::TenantsNotDict)?
        };
        for (name, value) in tenants.iter() {
            let name = name
                .as_str()
                .ok_or_else(|| ConfigFormatError::NameNotString(name.clone()))?;
            let base_dirs = parse_key_path_value_result(value, "base_dir", name)?;
            let extra_base_dirs = parse_key_path_value_result(value, "extra_base_dir", name)?;
            let mut extra_role_dirs = parse_key_path_value_result(value, "extra_role_dir", name)?;
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

        Ok(config)
    }

    fn filter_invalid_paths(config: Config) -> Config {
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
            tenant_config.is_tenant(work_dir).then_some(name.clone())
        })
    }

    pub fn get_tenant(&self, name: &str) -> Option<&TenantConfig> {
        self.tenants.get(name)
    }
}

fn resolve_config_path(custom_path: &Option<PathBuf>) -> PathBuf {
    match (custom_path, env::var("ZUUL_SEARCH_CONFIG_PATH")) {
        (Some(path), _) => path.clone(),
        (_, Ok(path)) => path.into(),
        (_, _) => dirs::config_dir().unwrap().join("zuul-ls/config.yaml"),
    }
}

fn make_common_roles_dir(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut ys = Vec::new();

    for name in ["zuul-shared", "zuul-trusted"] {
        ys.append(&mut base_dirs.iter().map(|x| x.join(name)).collect());
    }

    ys
}

fn parse_key_path_value_result(
    value: &Yaml,
    key: &str,
    tenant: &str,
) -> Result<Vec<PathBuf>, ParseConfigError> {
    let value =
        parse_key_path_value(value, key).ok_or_else(|| ConfigFormatError::ParseFieldError {
            tenant: tenant.into(),
            key: key.into(),
        })?;
    Ok(value)
}

fn parse_key_path_value(value: &Yaml, key: &str) -> Option<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_str() {
        let raw_str = r#"
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
            base_dirs: vec![PathBuf::from("~/foo/bar")],
            extra_base_dirs: vec![PathBuf::from("~/foo/another")],
            extra_role_dirs: vec![
                PathBuf::from("~/foo/another/extra_role"),
                PathBuf::from("~/foo/zar/extra-role2"),
                PathBuf::from("~/foo/bar/zuul-shared"),
                PathBuf::from("~/foo/bar/zuul-trusted"),
            ],
        };

        let config = Config {
            tenants: HashMap::from([("bar".into(), tenant)]),
        };

        assert_eq!(config, Config::parse_config_str(raw_str.into()).unwrap());
    }
}
