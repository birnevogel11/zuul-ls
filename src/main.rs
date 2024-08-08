mod config;
mod golden_key_test;
mod parser;
mod repo;
mod search;
mod study_graph;

use std::path::PathBuf;

use clap::Parser;

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
    crate::study_graph::study_graph_topo();
    // let args = ZuulSearchCli::parse();
    //
    // match args {
    //     ZuulSearchCli::Roles(args) => {
    //         crate::search::roles::list_roles_cli(args.work_dir, args.config_path);
    //     }
    //     ZuulSearchCli::Jobs(args) => {
    //         crate::search::jobs::list_jobs_cli(args.work_dir, args.config_path);
    //     }
    // };
}
