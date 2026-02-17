use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::commands;
use crate::config::AppContext;
use crate::models::{AgentKind, InstallTarget};

#[derive(Debug, Parser)]
#[command(
    name = "skillz",
    version,
    about = "Cross-agent skill manager for Codex and Claude"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    #[arg(long, global = true, value_enum)]
    pub agent: Option<AgentKind>,

    #[arg(long, global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "create", visible_alias = "Create")]
    Create(CreateArgs),

    #[command(name = "update", visible_alias = "Update")]
    Update(UpdateArgs),

    #[command(name = "delete", visible_alias = "Delete")]
    Delete(DeleteArgs),

    #[command(name = "list", visible_alias = "List")]
    List(ListArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub intent: Option<String>,

    #[arg(long, value_enum)]
    pub install: Option<InstallTarget>,

    #[arg(long)]
    pub alias: Option<String>,

    #[arg(long, value_enum)]
    pub agent: Option<AgentKind>,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub request: Option<String>,

    #[arg(long, value_enum)]
    pub agent: Option<AgentKind>,
}

#[derive(Debug, Args)]
pub struct DeleteArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args, Default)]
pub struct ListArgs {}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let ctx = AppContext::load(cli.root.clone())?;

    match cli.command {
        Commands::Create(args) => commands::create::run(&ctx, &args, cli.agent),
        Commands::Update(args) => commands::update::run(&ctx, &args, cli.agent),
        Commands::Delete(args) => commands::delete::run(&ctx, &args, cli.yes),
        Commands::List(args) => commands::list::run(&ctx, &args),
    }
}
