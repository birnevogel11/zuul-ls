mod config;
mod repo;
mod study_parser;
mod yaml_parse;

use crate::config::get_config;
use crate::study_parser::temp;
use std::path::Path;

// use clap::Parser;
//
//
// mod config;
//
// #[derive(Parser)]
// #[command(name = "zuul-search")]
// #[command(bin_name = "zuul-search")]
// enum ZuulSearchCli {
//     Roles(ZuulSearchCliRolesArgs),
// }
//
// #[derive(clap::Args)]
// #[command(version, about, long_about = "Search roles")]
// struct ZuulSearchCliRolesArgs {
//     #[arg(long)]
//     work_dir: Option<std::path::PathBuf>,
// }

fn main() {
    temp();
    println!("{:?}", get_config(&Some(&Path::new("./config.yaml"))));
    // let args = ZuulSearchCli::parse();
    //
    // match args {
    //     ZuulSearchCli::Roles(args) => {
    //         println!("{:?}", args.work_dir);
    //     }
    // };
}
