# Getting Started (No Rust Background)

This guide assumes you have never used Rust before.

## 1. Install prerequisites

- Install Rust (recommended: `rustup`)
- Confirm installation:

```bash
rustc --version
cargo --version
```

- Install the agent CLIs you plan to use:

```bash
codex --version
claude --version
```

If one command is missing, you can still use `skillz` with the other agent.

## 2. Build the project

From the repository root:

```bash
cargo build
```

What this does:

- Downloads dependencies
- Compiles the `skillz` binary

## 3. Verify CLI access

```bash
cargo run -- --help
```

You should see commands: `create`, `update`, `delete`, `list`.

## 4. List existing skills

```bash
cargo run -- list
```

On a fresh setup, this usually shows no skills.

## 5. Create your first skill

```bash
cargo run -- create
```

During create:

1. You enter a skill name and intent.
2. `skillz` launches interactive Codex or Claude session.
3. The agent generates a brief.
4. `skillz` creates canonical files under your root skill directory.
5. You choose where to install aliases (Codex, Claude, both, or skip).

## 6. Update a skill

```bash
cargo run -- update
```

You pick a skill, describe the change, and the chosen agent edits source files.
`skillz` then re-syncs installed aliases.

## 7. Delete a skill

```bash
cargo run -- delete
```

This removes:

- Canonical source folder
- Render artifacts
- Installed aliases

## 8. Run tests anytime

```bash
cargo test
```

This validates core behavior like path safety, state persistence, and generation logic.
