use yaml_rust2::yaml::Yaml;

use path_absolutize::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use dirs;
use yaml_rust2::yaml::YamlLoader;

extern crate shellexpand;

#[derive(Default, Debug, PartialEq)]
pub struct TenantConfig {
    name: String,
    base_dirs: Vec<PathBuf>,
    extra_base_dirs: Vec<PathBuf>,
    extra_role_dirs: Vec<PathBuf>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Config {
    default_tenant: String,
    tenants: HashMap<String, TenantConfig>,
}

fn to_path(x: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(x).into_owned())
        .absolutize()
        .unwrap()
        .into_owned()
}

fn expand_tilde_paths(xs: Vec<String>) -> Vec<PathBuf> {
    xs.iter().map(|x| to_path(x.as_str())).collect()
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
                let base_dirs = parse_str_or_list(&get_key_content(&t.1, "base_dir"));
                let extra_base_dirs = parse_str_or_list(&get_key_content(&t.1, "extra_base_dir"));
                let extra_role_dirs = parse_str_or_list(&get_key_content(&t.1, "extra_role_dir"));

                config.tenants.insert(
                    name.clone(),
                    TenantConfig {
                        name,
                        base_dirs: expand_tilde_paths(base_dirs),
                        extra_base_dirs: expand_tilde_paths(extra_base_dirs),
                        extra_role_dirs: expand_tilde_paths(extra_role_dirs),
                    },
                );
            }
        }
        Some(config)
    }

    fn read_config_from_path(custom_path: Option<PathBuf>) -> Option<Config> {
        match Self::read_config_file(custom_path) {
            Some(content) => Self::read_config_str(content),
            None => None,
        }
    }

    fn read_config_file(custom_path: Option<PathBuf>) -> Option<String> {
        let config_path = determine_config_path(custom_path);
        fs::read_to_string(config_path).ok()
    }
}

fn determine_config_path(custom_path: Option<PathBuf>) -> PathBuf {
    match (custom_path, env::var("ZUUL_STATIC_PARSER_CONFIG_PATH")) {
        (Some(path), _) => path,
        (_, Ok(path)) => path.into(),
        (_, _) => dirs::config_dir()
            .unwrap()
            .join("zuul-static-parser/config.yaml"),
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

fn get_key_content<'a>(raw_config: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    if let Some(raw_config) = raw_config.as_hash() {
        let search_key = Yaml::String(key.to_owned());
        return raw_config.get(&search_key);
    }
    None
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
            ],
            ..Default::default()
        };

        let config = Config {
            default_tenant: "bar".into(),
            tenants: HashMap::from([("bar".into(), tenant)]),
            ..Default::default()
        };

        assert_eq!(config, Config::read_config_str(raw_str.into()).unwrap());
    }
}
