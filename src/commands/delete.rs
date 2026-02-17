use anyhow::{Context, Result, bail};
use dialoguer::{Confirm, Select};
use std::fs;

use crate::cli::DeleteArgs;
use crate::config::AppContext;
use crate::install;
use crate::state;

pub fn run(ctx: &AppContext, args: &DeleteArgs, global_yes: bool) -> Result<()> {
    let skills = list_skills(ctx)?;
    if skills.is_empty() {
        bail!("no skills found in {}", ctx.skills_dir.display());
    }

    let skill_name = match &args.name {
        Some(name) => name.clone(),
        None => {
            let idx = Select::new()
                .with_prompt("Which skill do you want to delete?")
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

    let confirmed = if global_yes || args.yes {
        true
    } else {
        Confirm::new()
            .with_prompt(format!(
                "Delete skill '{}' from source and all aliases?",
                skill_name
            ))
            .default(false)
            .interact()
            .context("failed to read confirmation")?
    };

    if !confirmed {
        println!("Delete cancelled.");
        return Ok(());
    }

    let state_path = ctx.state_file();
    let mut install_state = state::load(&state_path)?;
    install::remove_skill_everywhere(ctx, &mut install_state, &skill_name)?;

    fs::remove_dir_all(&skill_dir)
        .with_context(|| format!("failed to remove {}", skill_dir.display()))?;

    state::save(&state_path, &install_state)?;

    println!("Deleted skill '{}' and all installed aliases.", skill_name);
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
