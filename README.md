# skillz

`skillz` is a Rust CLI for creating and managing cross-platform LLM skills used by Codex and Claude Code.

## Prerequisites

- Rust toolchain (`rustc`, `cargo`)
- `codex` CLI installed and available on `PATH` for Codex-driven create/update
- `claude` CLI installed and available on `PATH` for Claude-driven create/update

## Core model

- Canonical skill source lives in a configurable root directory.
- Default root: `~/.llm`
- Root resolution precedence: `--root` > `LLM_SKILLS_ROOT` > `~/.llm/config.toml` > `~/.llm`
- Canonical source directory: `<root>/skills/<skill-name>/`
- Platform render outputs:
  - `<root>/renders/codex/<alias>/`
  - `<root>/renders/claude/<alias>/`
- Installs are direct symlink aliases into:
  - Codex: `$CODEX_HOME/skills/<alias>` or `~/.codex/skills/<alias>`
  - Claude: `~/.claude/skills/<alias>`

## Command API

Commands are case-insensitive via aliases:

- `skillz create` / `skillz Create`
- `skillz update` / `skillz Update`
- `skillz delete` / `skillz Delete`
- `skillz list` / `skillz List`

## Command behavior

### Create

1. Prompt for skill name + intent.
2. Launch interactive `codex` or `claude` deep-dive session.
3. Agent writes a structured brief JSON to `<root>/work/.../skill-brief.json`.
4. `skillz` generates canonical source files from the brief.
5. Confirm with user.
6. Prompt install target (`codex|claude|both|skip`) and alias.
7. Render + install symlink aliases.

### Update

1. Select skill from canonical source.
2. Collect update request.
3. Launch interactive agent in the canonical skill folder.
4. Validate source skill structure.
5. Re-render and re-sync existing installs.

### Delete

1. Select skill.
2. Confirm.
3. Delete canonical source.
4. Remove render outputs and installed aliases.

### List

Shows all canonical skills and where each alias is installed.

## Build and run

```bash
cargo build
cargo run -- list
```

## Initial layout created by runtime

```
<root>/
  config.toml
  skills/
  renders/
    codex/
    claude/
  state/
    installs.json
  work/
```

## Current status

This v1 implementation focuses on local filesystem workflows and interactive CLI-driven creation/update. Platform-specific render transformations can be expanded in future iterations.
