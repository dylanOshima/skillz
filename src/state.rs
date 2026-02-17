use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::models::{InstallRecord, InstallState};

pub fn load(path: &Path) -> Result<InstallState> {
    if !path.exists() {
        return Ok(InstallState {
            version: 1,
            records: Vec::new(),
        });
    }

    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read state file: {}", path.display()))?;
    let mut state: InstallState = serde_json::from_str(&raw)
        .with_context(|| format!("invalid JSON in state file: {}", path.display()))?;
    if state.version == 0 {
        state.version = 1;
    }
    Ok(state)
}

pub fn save(path: &Path, state: &InstallState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directory for state file: {}",
                parent.display()
            )
        })?;
    }

    let raw =
        serde_json::to_string_pretty(state).context("failed to serialize install state to JSON")?;
    fs::write(path, raw)
        .with_context(|| format!("failed to write state file: {}", path.display()))?;
    Ok(())
}

pub fn upsert_record(state: &mut InstallState, updated: InstallRecord) {
    if let Some(existing) = state
        .records
        .iter_mut()
        .find(|r| r.skill_name == updated.skill_name && r.alias == updated.alias)
    {
        *existing = updated;
        return;
    }
    state.records.push(updated);
}

pub fn remove_skill_records(state: &mut InstallState, skill_name: &str) {
    state.records.retain(|r| r.skill_name != skill_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::models::InstallBinding;

    #[test]
    fn load_missing_file_returns_default_state() {
        let tmp = tempdir().expect("tempdir");
        let state_path = tmp.path().join("installs.json");

        let state = load(&state_path).expect("load should succeed");
        assert_eq!(state.version, 1);
        assert!(state.records.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempdir().expect("tempdir");
        let state_path = tmp.path().join("state").join("installs.json");

        let mut state = InstallState {
            version: 1,
            records: Vec::new(),
        };
        upsert_record(
            &mut state,
            InstallRecord {
                skill_name: "skill-a".to_string(),
                alias: "alias-a".to_string(),
                codex: Some(InstallBinding {
                    install_path: "/tmp/codex/alias-a".to_string(),
                    render_path: "/tmp/renders/codex/alias-a".to_string(),
                }),
                claude: None,
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            },
        );

        save(&state_path, &state).expect("save should pass");
        let loaded = load(&state_path).expect("load should pass");
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].skill_name, "skill-a");
        assert_eq!(loaded.records[0].alias, "alias-a");
    }

    #[test]
    fn upsert_updates_existing_record_and_remove_by_skill() {
        let mut state = InstallState {
            version: 1,
            records: Vec::new(),
        };

        upsert_record(
            &mut state,
            InstallRecord {
                skill_name: "skill-a".to_string(),
                alias: "alias-a".to_string(),
                codex: None,
                claude: None,
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            },
        );

        upsert_record(
            &mut state,
            InstallRecord {
                skill_name: "skill-a".to_string(),
                alias: "alias-a".to_string(),
                codex: Some(InstallBinding {
                    install_path: "/tmp/codex/alias-a".to_string(),
                    render_path: "/tmp/renders/codex/alias-a".to_string(),
                }),
                claude: None,
                updated_at: "2026-01-02T00:00:00Z".to_string(),
            },
        );

        assert_eq!(state.records.len(), 1);
        assert!(state.records[0].codex.is_some());

        remove_skill_records(&mut state, "skill-a");
        assert!(state.records.is_empty());
    }
}
