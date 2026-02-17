use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use crate::models::AgentKind;

pub fn run_create_session(agent: AgentKind, work_dir: &Path, prompt: &str) -> Result<()> {
    run(agent, work_dir, prompt)
}

pub fn run_update_session(agent: AgentKind, skill_dir: &Path, prompt: &str) -> Result<()> {
    run(agent, skill_dir, prompt)
}

fn run(agent: AgentKind, cwd: &Path, prompt: &str) -> Result<()> {
    let mut cmd = match agent {
        AgentKind::Codex => {
            let mut c = Command::new("codex");
            c.arg(prompt);
            c
        }
        AgentKind::Claude => {
            let mut c = Command::new("claude");
            c.arg(prompt);
            c
        }
    };

    let status = cmd
        .current_dir(cwd)
        .status()
        .with_context(|| format!("failed to start {}", agent.as_str()))?;

    if !status.success() {
        bail!("{} exited with status {}", agent.as_str(), status);
    }
    Ok(())
}
