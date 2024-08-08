use clap::Parser;

mod parse;

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
    work_dir: Option<std::path::PathBuf>,
}

fn main() {
    // let args = ZuulSearchCli::parse();
    //
    // match args {
    //     ZuulSearchCli::Roles(args) => {
    //         println!("{:?}", args.work_dir);
    //     }
    // };
}
