use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn normalize_skill_name(input: &str) -> String {
    let mut out = String::new();
    let mut prev_hyphen = false;

    for ch in input.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_hyphen = false;
        } else if !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    out
}

pub fn now_timestamp() -> String {
    Utc::now().to_rfc3339()
}

pub fn now_slug() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}

pub fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        bail!("source is not a directory: {}", from.display());
    }
    fs::create_dir_all(to)
        .with_context(|| format!("failed to create directory: {}", to.display()))?;

    for entry in fs::read_dir(from)
        .with_context(|| format!("failed to read directory: {}", from.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read entry in {}", from.display()))?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());

        let ty = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", source_path.display()))?;

        if ty.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if ty.is_file() {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        } else if ty.is_symlink() {
            let linked = fs::read_link(&source_path)
                .with_context(|| format!("failed to read symlink {}", source_path.display()))?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(linked, &target_path)
                .with_context(|| format!("failed to copy symlink to {}", target_path.display()))?;
        }
    }

    Ok(())
}

pub fn resolve_safe_path(base: &Path, relative: &str) -> Result<PathBuf> {
    let rel = Path::new(relative);
    if rel.is_absolute() {
        bail!("absolute paths are not allowed in resource entries: {relative}");
    }

    for comp in rel.components() {
        if matches!(
            comp,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            bail!("resource path attempts to escape skill directory: {relative}");
        }
    }

    Ok(base.join(rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn normalize_skill_name_collapses_invalid_chars() {
        let normalized = normalize_skill_name("  My Cool__Skill!! v2  ");
        assert_eq!(normalized, "my-cool-skill-v2");
    }

    #[test]
    fn normalize_skill_name_handles_empty_result() {
        let normalized = normalize_skill_name("___---___");
        assert_eq!(normalized, "");
    }

    #[test]
    fn resolve_safe_path_accepts_local_relative_paths() {
        let base = PathBuf::from("/tmp/base");
        let resolved = resolve_safe_path(&base, "scripts/tool.py").expect("path should resolve");
        assert_eq!(resolved, base.join("scripts/tool.py"));
    }

    #[test]
    fn resolve_safe_path_rejects_parent_dir_escape() {
        let base = PathBuf::from("/tmp/base");
        let err = resolve_safe_path(&base, "../evil.sh").expect_err("path should fail");
        assert!(err.to_string().contains("escape"));
    }

    #[test]
    fn resolve_safe_path_rejects_absolute_path() {
        let base = PathBuf::from("/tmp/base");
        let err = resolve_safe_path(&base, "/etc/passwd").expect_err("path should fail");
        assert!(err.to_string().contains("absolute"));
    }
}
