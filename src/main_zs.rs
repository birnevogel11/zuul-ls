use log;
use std::path::PathBuf;

use clap::Parser;

use zuul_parser::config::get_work_dir;
use zuul_parser::search::job_playbooks;
use zuul_parser::search::jobs;
use zuul_parser::search::project_templates;
use zuul_parser::search::roles;
use zuul_parser::search::vars;
use zuul_parser::search::work_dir_vars;

#[derive(Parser, Debug)]
#[command(name = "zuul-search")]
#[command(bin_name = "zuul-search")]
enum ZuulSearchCli {
    Roles(CliRolesArgs),
    Jobs(CliJobArgs),
    ProjectTemplates(CliProjectTemplateArgs),
    JobHierarchy(CliJobHierarchyArgs),
    JobVars(CliJobVariablesArgs),
    JobPlaybooks(CliJobPlaybooksArgs),
    WorkdirVars(CliWorkDirVarsArgs),
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "Search roles")]
struct CliRolesArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    #[arg(long, short)]
    local: bool,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List jobs")]
struct CliJobArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    #[arg(long, short)]
    local: bool,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List project templates")]
struct CliProjectTemplateArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    #[arg(long, short)]
    local: bool,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List job hierarchy of a job")]
struct CliJobHierarchyArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List variables in cwd")]
struct CliWorkDirVarsArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List variables of a job")]
struct CliJobVariablesArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = "List playbooks of a job")]
struct CliJobPlaybooksArgs {
    #[arg(long)]
    work_dir: Option<PathBuf>,

    #[arg(long)]
    config_path: Option<PathBuf>,

    name: String,
}

fn main() {
    env_logger::init();
    let args = ZuulSearchCli::parse();
    log::debug!("Parse args: {:#?}", args);

    match args {
        ZuulSearchCli::Roles(args) => {
            roles::list_roles_cli(&get_work_dir(args.work_dir), args.config_path, args.local);
        }
        ZuulSearchCli::Jobs(args) => {
            jobs::list_jobs_cli(&get_work_dir(args.work_dir), args.config_path, args.local);
        }
        ZuulSearchCli::ProjectTemplates(args) => {
            project_templates::list_project_templates_cli(
                &get_work_dir(args.work_dir),
                args.config_path,
                args.local,
            );
        }
        ZuulSearchCli::JobHierarchy(args) => {
            jobs::list_jobs_hierarchy_names_cli(
                args.name,
                &get_work_dir(args.work_dir),
                args.config_path,
            );
        }
        ZuulSearchCli::JobVars(args) => {
            vars::list_jobs_vars_cli(args.name, &get_work_dir(args.work_dir), args.config_path);
        }
        ZuulSearchCli::JobPlaybooks(args) => {
            job_playbooks::list_jobs_playbooks_cli(
                args.name,
                &get_work_dir(args.work_dir),
                args.config_path,
            );
        }
        ZuulSearchCli::WorkdirVars(args) => {
            work_dir_vars::list_work_dir_vars_cli(&get_work_dir(args.work_dir), args.config_path);
        }
    };
}
