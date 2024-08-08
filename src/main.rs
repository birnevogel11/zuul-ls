use std::path::Path;

use crate::config::get_config;
use crate::config::Config;

mod config;
mod repo;
mod yaml_parse;

fn main() {
    // let repo_dirs = list_repos(PathBuf::from("/home/yen3/carit"));
    // let role_dirs = list_roles(&repo_dirs);
    // println!("{:?}", repo_dirs);
    // println!("{:?}", role_dirs);
    //
    // let docs = YValueLoader::load_from_str("test: [1, 2, 3]").unwrap();
    // let doc = &docs[0]; // select the first YAML document
    // println!("{:?}", doc);

    // println!("{:?}", Config::read_config());
    // println!("{:?}", Config::read_config_path(Path::new("./config.yaml")));
    // println!(
    //     "{:?}",
    //     Config::validate_config(Config::read_config_path(Path::new("./config.yaml")).unwrap())
    // );
    // println!("{:?}", get_config(&Some(&Path::new("./config.yaml"))));
}
