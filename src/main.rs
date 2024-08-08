mod config;
mod parser;
mod repo;
mod search;
mod study_parser;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "zuul-search")]
#[command(bin_name = "zuul-search")]
enum ZuulSearchCli {
    Roles(ZuulSearchCliRolesArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = "Search roles")]
struct ZuulSearchCliRolesArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    #[arg()]
    search_key: Option<String>,
}

fn main() {
    let args = ZuulSearchCli::parse();

    match args {
        ZuulSearchCli::Roles(args) => {
            crate::search::roles::list_roles_cli(args.search_key, args.work_dir, args.config_path);
        }
    };
}
