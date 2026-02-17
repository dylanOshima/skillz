use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AppContext;
use crate::models::{AgentKind, InstallBinding, InstallRecord, InstallState};
use crate::renderer;
use crate::state;
use crate::utils::{now_slug, now_timestamp};

pub fn install_alias(
    ctx: &AppContext,
    state_obj: &mut InstallState,
    skill_name: &str,
    alias: &str,
    source_skill_dir: &Path,
    agent: AgentKind,
) -> Result<()> {
    let render_path = render_path(ctx, agent, alias);
    render_for_agent(agent, source_skill_dir, &render_path)?;

    let install_path = install_path(ctx, agent, alias);
    ensure_symlink(ctx, agent, alias, &render_path, &install_path)?;

    let mut record = state_obj
        .records
        .iter()
        .find(|r| r.skill_name == skill_name && r.alias == alias)
        .cloned()
        .unwrap_or_else(|| InstallRecord {
            skill_name: skill_name.to_string(),
            alias: alias.to_string(),
            codex: None,
            claude: None,
            updated_at: now_timestamp(),
        });

    let binding = InstallBinding {
        install_path: install_path.to_string_lossy().to_string(),
        render_path: render_path.to_string_lossy().to_string(),
    };

    match agent {
        AgentKind::Codex => record.codex = Some(binding),
        AgentKind::Claude => record.claude = Some(binding),
    }
    record.updated_at = now_timestamp();

    state::upsert_record(state_obj, record);
    Ok(())
}

pub fn sync_skill_installs(
    ctx: &AppContext,
    state_obj: &mut InstallState,
    skill_name: &str,
    source_skill_dir: &Path,
) -> Result<()> {
    for record in state_obj
        .records
        .iter_mut()
        .filter(|r| r.skill_name == skill_name)
    {
        if record.codex.is_some() {
            let alias = record.alias.clone();
            let render_path = render_path(ctx, AgentKind::Codex, &alias);
            render_for_agent(AgentKind::Codex, source_skill_dir, &render_path)?;
            let install_path = install_path(ctx, AgentKind::Codex, &alias);
            ensure_symlink(ctx, AgentKind::Codex, &alias, &render_path, &install_path)?;
            record.codex = Some(InstallBinding {
                install_path: install_path.to_string_lossy().to_string(),
                render_path: render_path.to_string_lossy().to_string(),
            });
        }

        if record.claude.is_some() {
            let alias = record.alias.clone();
            let render_path = render_path(ctx, AgentKind::Claude, &alias);
            render_for_agent(AgentKind::Claude, source_skill_dir, &render_path)?;
            let install_path = install_path(ctx, AgentKind::Claude, &alias);
            ensure_symlink(ctx, AgentKind::Claude, &alias, &render_path, &install_path)?;
            record.claude = Some(InstallBinding {
                install_path: install_path.to_string_lossy().to_string(),
                render_path: render_path.to_string_lossy().to_string(),
            });
        }

        record.updated_at = now_timestamp();
    }

    Ok(())
}

pub fn remove_skill_everywhere(
    ctx: &AppContext,
    state_obj: &mut InstallState,
    skill_name: &str,
) -> Result<()> {
    let records: Vec<InstallRecord> = state_obj
        .records
        .iter()
        .filter(|r| r.skill_name == skill_name)
        .cloned()
        .collect();

    for record in &records {
        if let Some(binding) = &record.codex {
            remove_binding(binding)?;
        }
        if let Some(binding) = &record.claude {
            remove_binding(binding)?;
        }

        let codex_render = render_path(ctx, AgentKind::Codex, &record.alias);
        let claude_render = render_path(ctx, AgentKind::Claude, &record.alias);

        if codex_render.exists() {
            fs::remove_dir_all(&codex_render).with_context(|| {
                format!(
                    "failed to remove render directory {}",
                    codex_render.display()
                )
            })?;
        }
        if claude_render.exists() {
            fs::remove_dir_all(&claude_render).with_context(|| {
                format!(
                    "failed to remove render directory {}",
                    claude_render.display()
                )
            })?;
        }
    }

    state::remove_skill_records(state_obj, skill_name);
    Ok(())
}

fn render_for_agent(agent: AgentKind, source_skill_dir: &Path, render_path: &Path) -> Result<()> {
    match agent {
        AgentKind::Codex => renderer::codex::render(source_skill_dir, render_path),
        AgentKind::Claude => renderer::claude::render(source_skill_dir, render_path),
    }
}

fn ensure_symlink(
    ctx: &AppContext,
    agent: AgentKind,
    alias: &str,
    target: &Path,
    link_path: &Path,
) -> Result<()> {
    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    if link_path.exists() || fs::symlink_metadata(link_path).is_ok() {
        let metadata = fs::symlink_metadata(link_path)
            .with_context(|| format!("failed to inspect {}", link_path.display()))?;
        if metadata.file_type().is_symlink() {
            fs::remove_file(link_path)
                .with_context(|| format!("failed to remove symlink {}", link_path.display()))?;
        } else {
            let backup_dir = ctx.state_dir.join("backups").join(agent.as_str());
            fs::create_dir_all(&backup_dir)
                .with_context(|| format!("failed to create backup dir {}", backup_dir.display()))?;
            let backup_path = backup_dir.join(format!("{}-{}", alias, now_slug()));
            fs::rename(link_path, &backup_path).with_context(|| {
                format!(
                    "failed to move existing path {} to backup {}",
                    link_path.display(),
                    backup_path.display()
                )
            })?;
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link_path).with_context(|| {
        format!(
            "failed to create symlink {} -> {}",
            link_path.display(),
            target.display()
        )
    })?;

    Ok(())
}

fn remove_binding(binding: &InstallBinding) -> Result<()> {
    let install_path = PathBuf::from(&binding.install_path);
    if install_path.exists() || fs::symlink_metadata(&install_path).is_ok() {
        let metadata = fs::symlink_metadata(&install_path)
            .with_context(|| format!("failed to inspect {}", install_path.display()))?;
        if metadata.file_type().is_symlink() {
            fs::remove_file(&install_path)
                .with_context(|| format!("failed to remove symlink {}", install_path.display()))?;
        }
    }
    Ok(())
}

fn render_path(ctx: &AppContext, agent: AgentKind, alias: &str) -> PathBuf {
    match agent {
        AgentKind::Codex => ctx.renders_codex_dir.join(alias),
        AgentKind::Claude => ctx.renders_claude_dir.join(alias),
    }
}

fn install_path(ctx: &AppContext, agent: AgentKind, alias: &str) -> PathBuf {
    match agent {
        AgentKind::Codex => ctx.codex_install_root().join(alias),
        AgentKind::Claude => ctx.claude_install_root().join(alias),
    }
}
