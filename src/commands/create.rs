use anyhow::{Context, Result, bail};
use dialoguer::{Confirm, Input, Select};
use std::fs;

use crate::agent;
use crate::cli::CreateArgs;
use crate::config::AppContext;
use crate::generator;
use crate::install;
use crate::models::{AgentKind, InstallTarget, SkillBrief};
use crate::state;
use crate::utils::{normalize_skill_name, now_slug};

const MAX_SKILL_NAME_LEN: usize = 64;

pub fn run(ctx: &AppContext, args: &CreateArgs, global_agent: Option<AgentKind>) -> Result<()> {
    let raw_name = match &args.name {
        Some(name) => name.clone(),
        None => Input::<String>::new()
            .with_prompt("Skill name")
            .interact_text()
            .context("failed to read skill name")?,
    };
    let skill_name = normalize_skill_name(&raw_name);
    validate_skill_name(&skill_name)?;

    let skill_dir = ctx.skills_dir.join(&skill_name);
    if skill_dir.exists() {
        bail!("skill already exists: {}", skill_dir.display());
    }

    let intent = match &args.intent {
        Some(intent) => intent.clone(),
        None => Input::<String>::new()
            .with_prompt("What should this skill do?")
            .interact_text()
            .context("failed to read skill intent")?,
    };

    let agent = global_agent.or(args.agent).unwrap_or(ctx.default_agent());

    let work_dir = ctx.work_dir.join(format!("{}-{}", now_slug(), skill_name));
    fs::create_dir_all(&work_dir)
        .with_context(|| format!("failed to create work directory: {}", work_dir.display()))?;

    let brief_path = work_dir.join("skill-brief.json");
    let prompt = build_create_prompt(&skill_name, &intent, &brief_path);
    agent::run_create_session(agent, &work_dir, &prompt)?;

    if !brief_path.exists() {
        bail!(
            "agent session completed but no brief was found at {}",
            brief_path.display()
        );
    }

    let raw_brief = fs::read_to_string(&brief_path)
        .with_context(|| format!("failed to read {}", brief_path.display()))?;
    let mut brief: SkillBrief = serde_json::from_str(&raw_brief)
        .with_context(|| format!("invalid JSON in {}", brief_path.display()))?;

    brief.name = skill_name.clone();
    validate_brief(&brief)?;

    generator::generate_from_brief(&skill_dir, &brief)?;

    println!("Created canonical skill at {}", skill_dir.display());
    println!("- sections: {}", brief.sections.len());
    println!("- scripts: {}", brief.scripts.len());
    println!("- references: {}", brief.references.len());
    println!("- assets: {}", brief.assets.len());

    let keep_going = Confirm::new()
        .with_prompt("Does this look good to continue with installation?")
        .default(true)
        .interact()
        .context("failed to read confirmation")?;

    if !keep_going {
        println!(
            "Skipped installation. Source remains at {}",
            skill_dir.display()
        );
        return Ok(());
    }

    let install_target = match args.install {
        Some(target) => target,
        None => prompt_install_target()?,
    };

    if install_target == InstallTarget::Skip {
        println!(
            "Skipped installation. Source remains at {}",
            skill_dir.display()
        );
        return Ok(());
    }

    let alias = match &args.alias {
        Some(alias) => normalize_skill_name(alias),
        None => {
            let input = Input::<String>::new()
                .with_prompt("Alias name for install")
                .default(skill_name.clone())
                .interact_text()
                .context("failed to read alias")?;
            normalize_skill_name(&input)
        }
    };
    validate_skill_name(&alias)?;

    let state_path = ctx.state_file();
    let mut install_state = state::load(&state_path)?;

    match install_target {
        InstallTarget::Codex => {
            install::install_alias(
                ctx,
                &mut install_state,
                &skill_name,
                &alias,
                &skill_dir,
                AgentKind::Codex,
            )?;
            println!("Installed alias '{}' for codex", alias);
        }
        InstallTarget::Claude => {
            install::install_alias(
                ctx,
                &mut install_state,
                &skill_name,
                &alias,
                &skill_dir,
                AgentKind::Claude,
            )?;
            println!("Installed alias '{}' for claude", alias);
        }
        InstallTarget::Both => {
            install::install_alias(
                ctx,
                &mut install_state,
                &skill_name,
                &alias,
                &skill_dir,
                AgentKind::Codex,
            )?;
            install::install_alias(
                ctx,
                &mut install_state,
                &skill_name,
                &alias,
                &skill_dir,
                AgentKind::Claude,
            )?;
            println!("Installed alias '{}' for codex + claude", alias);
        }
        InstallTarget::Skip => {}
    }

    state::save(&state_path, &install_state)?;
    Ok(())
}

fn validate_brief(brief: &SkillBrief) -> Result<()> {
    if brief.description.trim().is_empty() {
        bail!("brief description is required");
    }
    if brief.overview.trim().is_empty() {
        bail!("brief overview is required");
    }
    Ok(())
}

fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("name must contain at least one letter or digit");
    }
    if name.len() > MAX_SKILL_NAME_LEN {
        bail!(
            "name is too long ({}). max is {} characters",
            name.len(),
            MAX_SKILL_NAME_LEN
        );
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("name must be lowercase hyphen-case");
    }
    Ok(())
}

fn prompt_install_target() -> Result<InstallTarget> {
    let options = ["codex", "claude", "both", "skip"];
    let idx = Select::new()
        .with_prompt("Where should this skill be installed?")
        .items(&options)
        .default(2)
        .interact()
        .context("failed to choose install target")?;

    Ok(match idx {
        0 => InstallTarget::Codex,
        1 => InstallTarget::Claude,
        2 => InstallTarget::Both,
        _ => InstallTarget::Skip,
    })
}

fn build_create_prompt(skill_name: &str, intent: &str, brief_path: &std::path::Path) -> String {
    format!(
        "Create a cross-agent skill design brief.\n\
Skill name: {skill_name}\n\
Intent: {intent}\n\
Write ONLY valid JSON to this exact file path: {path}\n\
JSON schema:\n\
{{\n\
  \"name\": \"{skill_name}\",\n\
  \"description\": \"string\",\n\
  \"overview\": \"string\",\n\
  \"sections\": [{{\"title\":\"string\",\"body\":\"string\"}}],\n\
  \"scripts\": [{{\"path\":\"script.py\",\"content\":\"...\"}}],\n\
  \"references\": [{{\"path\":\"reference.md\",\"content\":\"...\"}}],\n\
  \"assets\": [{{\"path\":\"template.txt\",\"content\":\"...\"}}]\n\
}}\n\
Keep it concise and practical.",
        path = brief_path.display()
    )
}
