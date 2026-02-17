use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

use crate::models::{ResourceEntry, SkillBrief};
use crate::utils::resolve_safe_path;

pub fn generate_from_brief(skill_root: &Path, brief: &SkillBrief) -> Result<()> {
    if skill_root.exists() {
        bail!("skill directory already exists: {}", skill_root.display());
    }

    fs::create_dir_all(skill_root)
        .with_context(|| format!("failed to create skill dir: {}", skill_root.display()))?;

    let skill_md = render_skill_md(brief);
    fs::write(skill_root.join("SKILL.md"), skill_md)
        .with_context(|| format!("failed to write {}/SKILL.md", skill_root.display()))?;

    write_resource_entries(skill_root, "scripts", &brief.scripts)?;
    write_resource_entries(skill_root, "references", &brief.references)?;
    write_resource_entries(skill_root, "assets", &brief.assets)?;

    Ok(())
}

pub fn validate_skill_source(skill_dir: &Path) -> Result<()> {
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        bail!("SKILL.md missing in {}", skill_dir.display());
    }

    let raw = fs::read_to_string(&skill_md)
        .with_context(|| format!("failed to read {}", skill_md.display()))?;

    if !raw.starts_with("---\n") {
        bail!("SKILL.md must start with YAML frontmatter");
    }

    let Some(end_idx) = raw[4..].find("\n---\n") else {
        bail!("SKILL.md frontmatter is missing closing delimiter");
    };

    let frontmatter = &raw[4..4 + end_idx];
    let has_name = frontmatter
        .lines()
        .any(|line| line.trim_start().starts_with("name:"));
    let has_description = frontmatter
        .lines()
        .any(|line| line.trim_start().starts_with("description:"));

    if !has_name {
        bail!("SKILL.md frontmatter missing required key: name");
    }
    if !has_description {
        bail!("SKILL.md frontmatter missing required key: description");
    }

    Ok(())
}

pub fn write_resource_entries(
    skill_root: &Path,
    dir_name: &str,
    entries: &[ResourceEntry],
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let base = skill_root.join(dir_name);
    fs::create_dir_all(&base).with_context(|| format!("failed to create {}", base.display()))?;

    for entry in entries {
        let path = resolve_safe_path(&base, &entry.path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let content = if entry.content.trim().is_empty() {
            placeholder_content(dir_name, &entry.path)
        } else {
            entry.content.clone()
        };

        fs::write(&path, content)
            .with_context(|| format!("failed to write resource file: {}", path.display()))?;
    }

    Ok(())
}

fn render_skill_md(brief: &SkillBrief) -> String {
    let mut out = String::new();

    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", brief.name));
    out.push_str(&format!("description: {}\n", brief.description));
    out.push_str("---\n\n");

    out.push_str(&format!("# {}\n\n", title_case(&brief.name)));
    out.push_str("## Overview\n\n");
    out.push_str(brief.overview.trim());
    out.push_str("\n\n");

    for section in &brief.sections {
        out.push_str(&format!("## {}\n\n", section.title.trim()));
        out.push_str(section.body.trim());
        out.push_str("\n\n");
    }

    out
}

fn title_case(name: &str) -> String {
    name.split('-')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn placeholder_content(kind: &str, path: &str) -> String {
    format!(
        "# Placeholder\n\nThis file was generated for `{}` resource `{}`.\n",
        kind, path
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::models::{ResourceEntry, SkillBrief, SkillSection};

    fn sample_brief() -> SkillBrief {
        SkillBrief {
            name: "test-skill".to_string(),
            description: "Test skill description".to_string(),
            overview: "This is a test overview.".to_string(),
            sections: vec![SkillSection {
                title: "Usage".to_string(),
                body: "Use this skill for testing.".to_string(),
            }],
            scripts: vec![ResourceEntry {
                path: "helper.py".to_string(),
                content: "print('ok')\n".to_string(),
            }],
            references: vec![ResourceEntry {
                path: "guide.md".to_string(),
                content: "# Guide\n".to_string(),
            }],
            assets: vec![ResourceEntry {
                path: "template.txt".to_string(),
                content: String::new(),
            }],
        }
    }

    #[test]
    fn generate_from_brief_creates_expected_files() {
        let tmp = tempdir().expect("tempdir");
        let skill_root = tmp.path().join("test-skill");

        generate_from_brief(&skill_root, &sample_brief()).expect("generation should pass");
        validate_skill_source(&skill_root).expect("generated skill should validate");

        assert!(skill_root.join("SKILL.md").exists());
        assert!(skill_root.join("scripts").join("helper.py").exists());
        assert!(skill_root.join("references").join("guide.md").exists());
        assert!(skill_root.join("assets").join("template.txt").exists());
    }

    #[test]
    fn validate_skill_source_fails_without_frontmatter() {
        let tmp = tempdir().expect("tempdir");
        let skill_root = tmp.path().join("broken-skill");
        fs::create_dir_all(&skill_root).expect("create dir");
        fs::write(skill_root.join("SKILL.md"), "# Missing frontmatter\n").expect("write file");

        let err = validate_skill_source(&skill_root).expect_err("validation should fail");
        assert!(err.to_string().contains("frontmatter"));
    }

    #[test]
    fn write_resource_entries_rejects_escape_paths() {
        let tmp = tempdir().expect("tempdir");
        let skill_root = tmp.path().join("test-skill");
        fs::create_dir_all(&skill_root).expect("create dir");

        let entries = vec![ResourceEntry {
            path: "../escape.txt".to_string(),
            content: "bad".to_string(),
        }];

        let err =
            write_resource_entries(&skill_root, "scripts", &entries).expect_err("should fail");
        assert!(err.to_string().contains("escape"));
    }
}
