use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::{BuildArgs, InitArgs};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Harness {
    Claude,
    Codex,
    Opencode,
}

impl Harness {
    fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    version: u32,
    source: Harness,
    targets: Vec<Harness>,
    plugin: Plugin,
    paths: Paths,
    #[serde(default)]
    publish: Publish,
    #[serde(default)]
    policy: Policy,
}

#[derive(Debug, Deserialize, Serialize)]
struct Plugin {
    name: String,
    description: String,
    #[serde(default)]
    marketplace: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct Paths {
    source: PathBuf,
    skills: Option<PathBuf>,
}
#[derive(Debug, Default, Deserialize, Serialize)]
struct Publish {
    #[serde(default = "default_branch")]
    branch: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct Policy {
    #[serde(default = "default_unsupported")]
    unsupported: String,
}

fn default_branch() -> String {
    "skillz-build".into()
}
fn default_unsupported() -> String {
    "warn".into()
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            unsupported: default_unsupported(),
        }
    }
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    source: &'a str,
    targets: Vec<&'a str>,
    compatibility: &'a str,
    diagnostics: Vec<String>,
}

pub fn init(args: &InitArgs) -> Result<()> {
    let cwd = env::current_dir().context("failed to determine project directory")?;
    let name = args.name.clone().unwrap_or_else(|| {
        cwd.file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("plugin")
            .to_string()
    });
    validate_name(&name)?;
    let config_path = cwd.join("skillz.yaml");
    let workflow_path = cwd.join(".github/workflows/skillz.yml");
    if (!args.force && config_path.exists()) || (!args.force && workflow_path.exists()) {
        bail!(
            "skillz.yaml or .github/workflows/skillz.yml already exists; use --force to replace it"
        )
    }
    let skills = args
        .skills_path
        .clone()
        .or_else(|| default_skills_path(&cwd, args.source));
    let config = Config {
        version: 1,
        source: args.source,
        targets: vec![Harness::Claude, Harness::Codex, Harness::Opencode],
        plugin: Plugin {
            name: name.clone(),
            description: args
                .description
                .clone()
                .unwrap_or_else(|| format!("{} cross-harness plugin", name)),
            marketplace: Some(format!("{}-skillz", name)),
        },
        paths: Paths {
            source: args.source_path.clone(),
            skills,
        },
        publish: Publish {
            branch: default_branch(),
        },
        policy: Policy {
            unsupported: default_unsupported(),
        },
    };
    fs::write(
        &config_path,
        serde_yaml::to_string(&config).context("serialize skillz.yaml")?,
    )
    .with_context(|| format!("write {}", config_path.display()))?;
    fs::create_dir_all(workflow_path.parent().expect("workflow parent"))?;
    fs::write(&workflow_path, workflow_template())?;
    append_gitignore(&cwd.join(".gitignore"))?;
    fs::write(
        cwd.join("INSTALL.md"),
        install_doc(&config, repository_slug(&cwd)),
    )?;
    println!("Added Skillz support to {}", cwd.display());
    println!("  source: {}", args.source.as_str());
    println!("  config: {}", config_path.display());
    println!("  workflow: {}", workflow_path.display());
    Ok(())
}

pub fn build(args: &BuildArgs) -> Result<()> {
    let cwd = env::current_dir().context("failed to determine project directory")?;
    let config_path = cwd.join(&args.config);
    let config: Config = serde_yaml::from_str(
        &fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?,
    )
    .with_context(|| format!("parse {}", config_path.display()))?;
    validate_config(&config, &cwd)?;
    let output = absolute(&cwd, &args.output);
    if output == cwd || output == cwd.join(&config.paths.source) {
        bail!("output directory must not overlap the project or source directory")
    }
    if output.exists() {
        fs::remove_dir_all(&output).with_context(|| format!("clear {}", output.display()))?;
    }
    fs::create_dir_all(&output)?;
    let source = cwd.join(&config.paths.source);
    let skills = config.paths.skills.as_ref().map(|p| cwd.join(p));
    let mut diagnostics = Vec::new();
    for target in &config.targets {
        // OpenCode V1 installs Git packages from the repository root. Keeping the
        // package there also lets Claude/Codex marketplace catalogs reference
        // their own subdirectories without a second distribution repository.
        let target_dir = if *target == Harness::Opencode {
            output.clone()
        } else {
            output.join(target.as_str())
        };
        match target {
            Harness::Opencode if config.source == Harness::Opencode => {
                copy_opencode(&source, &target_dir)?
            }
            Harness::Claude => render_skill_plugin(
                &config,
                skills.as_deref(),
                &target_dir,
                Harness::Claude,
                &mut diagnostics,
            )?,
            Harness::Codex => render_skill_plugin(
                &config,
                skills.as_deref(),
                &target_dir,
                Harness::Codex,
                &mut diagnostics,
            )?,
            Harness::Opencode => {
                render_opencode_shell(&config, skills.as_deref(), &target_dir, &mut diagnostics)?
            }
        }
    }
    write_marketplaces(&config, &output)?;
    let compatibility = if diagnostics.is_empty() {
        "portable"
    } else {
        "adapter-required"
    };
    if config.policy.unsupported == "error" && !diagnostics.is_empty() {
        bail!("target overrides are required: {}", diagnostics.join("; "))
    }
    let report = Report {
        version: 1,
        source: config.source.as_str(),
        targets: config.targets.iter().map(|x| x.as_str()).collect(),
        compatibility,
        diagnostics,
    };
    fs::write(
        output.join("skillz-report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!(
        "Built {} targets in {} ({})",
        config.targets.len(),
        output.display(),
        compatibility
    );
    Ok(())
}

fn render_skill_plugin(
    config: &Config,
    skills: Option<&Path>,
    target: &Path,
    harness: Harness,
    diagnostics: &mut Vec<String>,
) -> Result<()> {
    let skills = skills.context("paths.skills is required to build Claude or Codex outputs")?;
    copy_dir(skills, &target.join("skills"), &BTreeSet::new())?;
    let manifest_dir = match harness {
        Harness::Claude => ".claude-plugin",
        Harness::Codex => ".codex-plugin",
        Harness::Opencode => unreachable!(),
    };
    fs::create_dir_all(target.join(manifest_dir))?;
    let manifest = serde_json::json!({ "name": config.plugin.name, "description": config.plugin.description, "skills": "./skills" });
    fs::write(
        target.join(manifest_dir).join("plugin.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    if config.source == Harness::Opencode {
        diagnostics.push(format!(
            "{} runtime, agents and commands require a {} override",
            config.source.as_str(),
            harness.as_str()
        ));
    }
    Ok(())
}

fn render_opencode_shell(
    config: &Config,
    skills: Option<&Path>,
    target: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<()> {
    let skills = skills.context("paths.skills is required to build an OpenCode output")?;
    copy_dir(skills, &target.join("skills"), &BTreeSet::new())?;
    let package = serde_json::json!({ "name": format!("opencode-{}", config.plugin.name), "version": "0.0.0-skillz", "type": "module", "files": ["skills"], "skillz": { "generated": true } });
    fs::write(
        target.join("package.json"),
        serde_json::to_string_pretty(&package)?,
    )?;
    diagnostics.push(format!(
        "{} source needs an OpenCode runtime override",
        config.source.as_str()
    ));
    Ok(())
}

fn copy_opencode(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for name in ["package.json", "README.md", "LICENSE", "opencode.json"] {
        let from = source.join(name);
        if from.is_file() {
            fs::copy(&from, target.join(name))?;
        }
    }
    let ignored = BTreeSet::from([
        "node_modules".to_string(),
        ".git".to_string(),
        ".github".to_string(),
        ".skillz".to_string(),
    ]);
    for name in ["dist", "skills", "agents", "commands", "scripts", "tools"] {
        let from = source.join(name);
        if from.is_dir() {
            copy_dir(&from, &target.join(name), &ignored)?;
        }
    }
    Ok(())
}

fn write_marketplaces(config: &Config, output: &Path) -> Result<()> {
    let marketplace = config
        .plugin
        .marketplace
        .clone()
        .unwrap_or_else(|| format!("{}-skillz", config.plugin.name));
    fs::create_dir_all(output.join(".claude-plugin"))?;
    fs::create_dir_all(output.join(".agents/plugins"))?;
    let claude = serde_json::json!({"name": marketplace, "owner": {"name": "Skillz"}, "plugins": [{"name": config.plugin.name, "description": config.plugin.description, "source": "./claude"}]});
    let codex = serde_json::json!({"name": marketplace, "plugins": [{"name": config.plugin.name, "source": {"source": "local", "path": "./codex"}}]});
    fs::write(
        output.join(".claude-plugin/marketplace.json"),
        serde_json::to_string_pretty(&claude)?,
    )?;
    fs::write(
        output.join(".agents/plugins/marketplace.json"),
        serde_json::to_string_pretty(&codex)?,
    )?;
    Ok(())
}

fn copy_dir(from: &Path, to: &Path, ignored: &BTreeSet<String>) -> Result<()> {
    if !from.is_dir() {
        bail!("source directory not found: {}", from.display())
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if ignored.contains(name_s.as_ref()) {
            continue;
        }
        let dst = to.join(&name);
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(&entry.path(), &dst, ignored)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), dst)?;
        }
    }
    Ok(())
}

fn validate_config(config: &Config, cwd: &Path) -> Result<()> {
    if config.version != 1 {
        bail!("unsupported skillz config version: {}", config.version)
    }
    validate_name(&config.plugin.name)?;
    if config.targets.is_empty() {
        bail!("targets must not be empty")
    }
    if !cwd.join(&config.paths.source).is_dir() {
        bail!(
            "source path not found: {}",
            cwd.join(&config.paths.source).display()
        )
    }
    Ok(())
}

fn validate_name(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("plugin name must be lowercase hyphen-case")
    }
    Ok(())
}

fn absolute(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
fn default_skills_path(cwd: &Path, source: Harness) -> Option<PathBuf> {
    let candidates: &[&str] = if source == Harness::Opencode {
        &["dist/skills", "skills"]
    } else {
        &["skills"]
    };
    candidates
        .iter()
        .find(|p| cwd.join(p).is_dir())
        .map(PathBuf::from)
}
fn append_gitignore(path: &Path) -> Result<()> {
    let rule = ".skillz/\n";
    let old = fs::read_to_string(path).unwrap_or_default();
    if !old.lines().any(|x| x.trim() == ".skillz/") {
        fs::write(
            path,
            format!(
                "{}{}",
                old,
                if old.ends_with('\n') || old.is_empty() {
                    rule.to_string()
                } else {
                    format!("\n{}", rule)
                }
            ),
        )?;
    }
    Ok(())
}
fn repository_slug(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8(output.stdout).ok()?;
    let normalized = url.trim().trim_end_matches(".git");
    normalized
        .rsplit_once("github.com/")
        .or_else(|| normalized.rsplit_once("github.com:"))
        .map(|(_, slug)| slug.to_string())
}

fn install_doc(config: &Config, repository: Option<String>) -> String {
    let repository = repository.unwrap_or_else(|| "OWNER/REPO".to_string());
    format!(
        "# Install {}\n\nAfter the first push, install the generated build from the `{}` branch.\n\n- Claude: `claude plugin marketplace add {}@{}` then `claude plugin install {}@{}`\n- Codex: `codex plugin marketplace add {} --ref {}` then `codex plugin add {}@{}`.\n- OpenCode: `opencode plugin add 'github:{}#{}' --global`\n",
        config.plugin.name,
        config.publish.branch,
        repository,
        config.publish.branch,
        config.plugin.name,
        config.plugin.marketplace.clone().unwrap_or_default(),
        repository,
        config.publish.branch,
        config.plugin.name,
        config.plugin.marketplace.clone().unwrap_or_default(),
        repository,
        config.publish.branch
    )
}
fn workflow_template() -> &'static str {
    r#"name: Skillz builds
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dylanOshima/skillz@v1
        with:
          mode: build
          output: .skillz/build
      - uses: actions/upload-artifact@v4
        with:
          name: skillz-build
          path: .skillz/build
  publish:
    if: github.event_name == 'push'
    needs: build
    permissions:
      contents: write
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dylanOshima/skillz@v1
        with:
          mode: publish
          output: .skillz/build
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn opencode_build_produces_three_install_roots() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("dist/skills/example")).unwrap();
        fs::write(
            root.join("dist/skills/example/SKILL.md"),
            "---\nname: example\ndescription: test\n---\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::write(root.join("dist/index.js"), "export {};\n").unwrap();
        fs::write(root.join("package.json"), "{\"name\":\"example\"}").unwrap();
        fs::write(root.join("skillz.yaml"), "version: 1\nsource: opencode\ntargets: [claude, codex, opencode]\nplugin:\n  name: example\n  description: Example\npaths:\n  source: .\n  skills: dist/skills\npolicy:\n  unsupported: warn\n").unwrap();
        let old = env::current_dir().unwrap();
        env::set_current_dir(root).unwrap();
        build(&BuildArgs {
            config: "skillz.yaml".into(),
            output: "out".into(),
        })
        .unwrap();
        env::set_current_dir(old).unwrap();
        assert!(root.join("out/claude/.claude-plugin/plugin.json").exists());
        assert!(root.join("out/codex/.codex-plugin/plugin.json").exists());
        assert!(root.join("out/package.json").exists());
    }
}
