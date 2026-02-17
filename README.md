# skillz

`skillz` is a command-line tool for creating and managing reusable AI skills for both Codex and Claude Code.

You do not need to know Rust to use it.

## What this project solves

Without `skillz`, you usually end up maintaining separate skill copies for each agent. `skillz` gives you:

- One canonical source folder for each skill.
- Alias-based installation into Codex and/or Claude.
- Update once, sync everywhere.

## If you are new to Rust

Rust is just the language this tool is written in.

To use this project, the main commands you need are:

- `cargo build`: compile the project.
- `cargo run -- <args>`: run the CLI without installing it globally.
- `cargo test`: run automated tests.

You do not need to write Rust code to use `skillz` as a CLI.

## Prerequisites

- Rust toolchain installed (`rustc`, `cargo`)
- `codex` CLI installed on your `PATH` if you want Codex-powered create/update sessions
- `claude` CLI installed on your `PATH` if you want Claude-powered create/update sessions

## Quick start (first 5 minutes)

1. Build once:

```bash
cargo build
```

2. Confirm the CLI is wired:

```bash
cargo run -- --help
```

3. See current skills (empty is fine):

```bash
cargo run -- list
```

4. Create your first skill:

```bash
cargo run -- create
```

5. Run tests:

```bash
cargo test
```

## Everyday usage

- Create a new skill: `cargo run -- create`
- Update an existing skill: `cargo run -- update`
- Delete a skill: `cargo run -- delete`
- List skills and install locations: `cargo run -- list`

Command aliases are also supported:

- `Create`, `Update`, `Delete`, `List`

## Where files are stored

Default root directory: `~/.llm`

Resolution precedence for the root directory:

1. `--root <path>`
2. `LLM_SKILLS_ROOT` environment variable
3. `~/.llm/config.toml`
4. `~/.llm`

Main layout:

```text
<root>/
  config.toml
  skills/               # canonical source skills
  renders/
    codex/              # generated codex install artifacts
    claude/             # generated claude install artifacts
  state/
    installs.json       # alias/install mapping state
  work/                 # temp working dirs for create sessions
```

Install targets:

- Codex: `$CODEX_HOME/skills/<alias>` or `~/.codex/skills/<alias>`
- Claude: `~/.claude/skills/<alias>`

## Docs

- Beginner setup: `docs/getting-started.md`
- Command guide: `docs/command-reference.md`
- Troubleshooting: `docs/troubleshooting.md`
- Specs:
  - `specs/cross-agent-authoring-spec.md`
  - `specs/codex-skills-spec.md`
  - `specs/claude-code-skills-spec.md`

## Current scope

v1 is local-filesystem focused:

- Interactive agent-driven create/update
- Canonical source + direct-copy render adapters
- Symlink alias installation for Codex/Claude
