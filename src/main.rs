mod agent;
mod cli;
mod commands;
mod config;
mod generator;
mod install;
mod models;
mod renderer;
mod state;
mod utils;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
