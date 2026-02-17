use anyhow::{Context, Result, bail};
use dialoguer::{Input, Select};
use std::fs;

use crate::agent;
use crate::cli::UpdateArgs;
use crate::config::AppContext;
use crate::generator;
use crate::install;
use crate::models::AgentKind;
use crate::state;

pub fn run(ctx: &AppContext, args: &UpdateArgs, global_agent: Option<AgentKind>) -> Result<()> {
    let skills = list_skills(ctx)?;
    if skills.is_empty() {
        bail!("no skills found in {}", ctx.skills_dir.display());
    }

    let skill_name = match &args.name {
        Some(name) => name.clone(),
        None => {
            let idx = Select::new()
                .with_prompt("Which skill do you want to update?")
                .items(&skills)
                .default(0)
                .interact()
                .context("failed to select skill")?;
            skills[idx].clone()
        }
    };

    let skill_dir = ctx.skills_dir.join(&skill_name);
    if !skill_dir.exists() {
        bail!("skill does not exist: {}", skill_dir.display());
    }

    let request = match &args.request {
        Some(req) => req.clone(),
        None => Input::<String>::new()
            .with_prompt("What do you want to change?")
            .interact_text()
            .context("failed to read update request")?,
    };

    let agent = global_agent.or(args.agent).unwrap_or(ctx.default_agent());
    let prompt = format!(
        "Update this skill in-place based on the request below.\n\
Skill directory: {}\n\
Request: {}\n\
Edit the canonical source files directly in this folder.\n\
Ensure SKILL.md keeps frontmatter with required name and description.",
        skill_dir.display(),
        request
    );

    agent::run_update_session(agent, &skill_dir, &prompt)?;
    generator::validate_skill_source(&skill_dir)?;

    let state_path = ctx.state_file();
    let mut install_state = state::load(&state_path)?;
    install::sync_skill_installs(ctx, &mut install_state, &skill_name, &skill_dir)?;
    state::save(&state_path, &install_state)?;

    println!("Updated source skill at {}", skill_dir.display());
    println!("Re-synced installed aliases for {}", skill_name);
    Ok(())
}

fn list_skills(ctx: &AppContext) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(&ctx.skills_dir)
        .with_context(|| format!("failed to read {}", ctx.skills_dir.display()))?
    {
        let entry = entry.with_context(|| "failed to read skill directory entry")?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}
