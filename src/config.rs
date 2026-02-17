use anyhow::{Context, Result};
use dirs::home_dir;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{AgentKind, RootConfigFile};

#[derive(Debug, Clone)]
pub struct AppContext {
    pub root: PathBuf,
    pub skills_dir: PathBuf,
    pub renders_codex_dir: PathBuf,
    pub renders_claude_dir: PathBuf,
    pub state_dir: PathBuf,
    pub work_dir: PathBuf,
    pub config: RootConfigFile,
}

impl AppContext {
    pub fn load(root_override: Option<PathBuf>) -> Result<Self> {
        let default_root = home_dir()
            .context("failed to resolve home directory")?
            .join(".llm");

        let root_from_default_cfg = read_config_file(&default_root.join("config.toml"))?
            .and_then(|cfg| cfg.root)
            .map(PathBuf::from);

        let root = if let Some(r) = root_override {
            r
        } else if let Ok(r) = env::var("LLM_SKILLS_ROOT") {
            PathBuf::from(r)
        } else if let Some(r) = root_from_default_cfg {
            r
        } else {
            default_root
        };

        let config_path = root.join("config.toml");
        let config = read_config_file(&config_path)?.unwrap_or_default();

        let ctx = Self {
            skills_dir: root.join("skills"),
            renders_codex_dir: root.join("renders").join("codex"),
            renders_claude_dir: root.join("renders").join("claude"),
            state_dir: root.join("state"),
            work_dir: root.join("work"),
            root,
            config,
        };

        ctx.ensure_layout()?;
        Ok(ctx)
    }

    pub fn default_agent(&self) -> AgentKind {
        self.config.default_agent.unwrap_or(AgentKind::Codex)
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_dir.join("installs.json")
    }

    pub fn codex_install_root(&self) -> PathBuf {
        if let Ok(codex_home) = env::var("CODEX_HOME") {
            PathBuf::from(codex_home).join("skills")
        } else {
            home_dir()
                .unwrap_or_else(|| PathBuf::from("~"))
                .join(".codex")
                .join("skills")
        }
    }

    pub fn claude_install_root(&self) -> PathBuf {
        home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".claude")
            .join("skills")
    }

    fn ensure_layout(&self) -> Result<()> {
        for dir in [
            &self.root,
            &self.skills_dir,
            &self.renders_codex_dir,
            &self.renders_claude_dir,
            &self.state_dir,
            &self.work_dir,
        ] {
            fs::create_dir_all(dir)
                .with_context(|| format!("failed to create directory: {}", dir.display()))?;
        }
        Ok(())
    }
}

fn read_config_file(path: &Path) -> Result<Option<RootConfigFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let parsed: RootConfigFile =
        toml::from_str(&raw).with_context(|| format!("invalid TOML in {}", path.display()))?;
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_with_root_override_creates_layout() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path().join("llm-root");

        let ctx = AppContext::load(Some(root.clone())).expect("context should load");

        assert_eq!(ctx.root, root);
        assert!(ctx.skills_dir.exists());
        assert!(ctx.renders_codex_dir.exists());
        assert!(ctx.renders_claude_dir.exists());
        assert!(ctx.state_dir.exists());
        assert!(ctx.work_dir.exists());
        assert_eq!(ctx.default_agent(), AgentKind::Codex);
    }

    #[test]
    fn load_with_root_override_reads_config_file() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path().join("llm-root");
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("config.toml"), "default_agent = \"claude\"\n").expect("write config");

        let ctx = AppContext::load(Some(root)).expect("context should load");
        assert_eq!(ctx.default_agent(), AgentKind::Claude);
    }
}
