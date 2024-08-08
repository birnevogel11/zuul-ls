use std::path::PathBuf;

use clap::Parser;

use zuul_parser::config::get_work_dir;
use zuul_parser::search::jobs;
use zuul_parser::search::roles;

#[derive(Parser)]
#[command(name = "zuul-search")]
#[command(bin_name = "zuul-search")]
enum ZuulSearchCli {
    Roles(ZuulSearchCliRolesArgs),
    Jobs(ZuulSearchCliJobArgs),
    ListJobHierarchy(ZuulSearchCliJobHierarchyArgs),
    ListJobVars(ZuulSearchCliJobVariablesArgs),
    ListJobPlaybooks(ZuulSearchCliJobPlaybooksArgs),
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

#[derive(clap::Args)]
#[command(version, about, long_about = "Search jobs")]
struct ZuulSearchCliJobHierarchyArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

#[derive(clap::Args)]
#[command(version, about, long_about = "list variables")]
struct ZuulSearchCliJobVariablesArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

#[derive(clap::Args)]
#[command(version, about, long_about = "list playbooks")]
struct ZuulSearchCliJobPlaybooksArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

fn main() {
    let args = ZuulSearchCli::parse();

    match args {
        ZuulSearchCli::Roles(args) => {
            roles::list_roles_cli(&get_work_dir(args.work_dir), args.config_path);
        }
        ZuulSearchCli::Jobs(args) => {
            jobs::list_jobs_cli(&get_work_dir(args.work_dir), args.config_path);
        }
        ZuulSearchCli::ListJobHierarchy(args) => {
            jobs::list_jobs_hierarchy_names_cli(
                args.name,
                &get_work_dir(args.work_dir),
                args.config_path,
            );
        }
        ZuulSearchCli::ListJobVars(args) => {
            jobs::list_jobs_vars_cli(args.name, &get_work_dir(args.work_dir), args.config_path);
        }
        ZuulSearchCli::ListJobPlaybooks(args) => {
            jobs::list_jobs_playbooks_cli(
                args.name,
                &get_work_dir(args.work_dir),
                args.config_path,
            );
        }
    };
}
