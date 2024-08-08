mod repo;

use crate::repo::list_repos;
use crate::repo::list_roles;

use std::path::PathBuf;

fn main() {
    let repo_dirs = list_repos(PathBuf::from("/home/yen3"));
    let role_dirs = list_roles(&repo_dirs);
    println!("{:?}", repo_dirs);
    println!("{:?}", role_dirs);
}
