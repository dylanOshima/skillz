use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::cli::ListArgs;
use crate::config::AppContext;
use crate::state;

pub fn run(ctx: &AppContext, _args: &ListArgs) -> Result<()> {
    let mut skills = list_skills(ctx)?;
    skills.sort();

    let state_path = ctx.state_file();
    let install_state = state::load(&state_path)?;

    if skills.is_empty() {
        println!("No skills found in {}", ctx.skills_dir.display());
        return Ok(());
    }

    for skill in skills {
        println!("{}", skill);
        println!("  source: {}", ctx.skills_dir.join(&skill).display());

        let records: Vec<_> = install_state
            .records
            .iter()
            .filter(|r| r.skill_name == skill)
            .collect();

        if records.is_empty() {
            println!("  installs: none");
            continue;
        }

        for record in records {
            println!("  alias: {}", record.alias);
            if let Some(binding) = &record.codex {
                let broken = symlink_broken(Path::new(&binding.install_path));
                println!(
                    "    codex: {}{}",
                    binding.install_path,
                    if broken { " (broken)" } else { "" }
                );
            }
            if let Some(binding) = &record.claude {
                let broken = symlink_broken(Path::new(&binding.install_path));
                println!(
                    "    claude: {}{}",
                    binding.install_path,
                    if broken { " (broken)" } else { "" }
                );
            }
        }
    }

    Ok(())
}

fn symlink_broken(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            fs::read_link(path).map_or(true, |target| !target.exists())
        }
        Ok(_) => false,
        Err(_) => true,
    }
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
    Ok(names)
}
