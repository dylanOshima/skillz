use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::utils::copy_dir_recursive;

pub fn render(source_skill_dir: &Path, render_dir: &Path) -> Result<()> {
    if render_dir.exists() {
        fs::remove_dir_all(render_dir).with_context(|| {
            format!(
                "failed to clear existing render dir {}",
                render_dir.display()
            )
        })?;
    }
    copy_dir_recursive(source_skill_dir, render_dir)?;
    Ok(())
}
