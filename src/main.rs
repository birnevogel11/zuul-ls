use std::path::PathBuf;

use clap::Parser;

use zuul_parser::search::roles;
use zuul_parser::search::jobs;

#[derive(Parser)]
#[command(name = "zuul-search")]
#[command(bin_name = "zuul-search")]
enum ZuulSearchCli {
    Roles(ZuulSearchCliRolesArgs),
    Jobs(ZuulSearchCliJobArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = "Search roles")]
struct ZuulSearchCliRolesArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = "Search jobs")]
struct ZuulSearchCliJobArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,
}

fn main() {
    let args = ZuulSearchCli::parse();

    match args {
        ZuulSearchCli::Roles(args) => {
            roles::list_roles_cli(args.work_dir, args.config_path);
        }
        ZuulSearchCli::Jobs(args) => {
            jobs::list_jobs_cli(args.work_dir, args.config_path);
        }
    };
}
