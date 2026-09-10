use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::project::{self, Harness};

#[derive(Debug, Parser)]
#[command(
    name = "skillz",
    version,
    about = "Cross-harness plugin onboarding and build tool"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add Skillz configuration and a GitHub workflow to a plugin repository.
    Init(InitArgs),
    /// Render all enabled harness packages from the configured source tree.
    Build(BuildArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Native harness that owns the source files.
    #[arg(long, value_enum)]
    pub source: Harness,
    /// Plugin display/installation name. Defaults to the current directory name.
    #[arg(long)]
    pub name: Option<String>,
    /// Short plugin description used in generated manifests.
    #[arg(long)]
    pub description: Option<String>,
    /// Path to the native source root, relative to the project directory.
    #[arg(long, default_value = ".")]
    pub source_path: PathBuf,
    /// Path to portable skills, relative to the project directory.
    #[arg(long)]
    pub skills_path: Option<PathBuf>,
    /// Overwrite a pre-existing skillz.yaml or workflow.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long, default_value = "skillz.yaml")]
    pub config: PathBuf,
    /// Generated output directory. CI sets this to a temporary directory.
    #[arg(long, default_value = ".skillz/build")]
    pub output: PathBuf,
}

pub fn run() -> Result<()> {
    match Cli::parse().command {
        Commands::Init(args) => project::init(&args),
        Commands::Build(args) => project::build(&args),
    }
}
