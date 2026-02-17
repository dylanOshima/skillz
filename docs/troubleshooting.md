# Troubleshooting

## `cargo` or `rustc` not found

Symptom:

- `zsh: command not found: cargo`

Fix:

1. Install Rust via `rustup`.
2. Load cargo env in your shell:

```bash
. "$HOME/.cargo/env"
```

3. Add that line to `~/.zshrc` for persistence.

## `codex` or `claude` command not found

`skillz create` and `skillz update` launch these CLIs directly.

Fix:

- Install the missing CLI.
- Confirm with `codex --version` or `claude --version`.
- Or run with `--agent` for whichever is installed.

## `create` ends but brief file is missing

Symptom:

- Error about missing `skill-brief.json` in `<root>/work/...`.

Cause:

- The interactive agent session ended before writing the required JSON file.

Fix:

- Re-run `create`.
- In the session, explicitly ensure the agent writes the JSON to the provided path.

## Permission denied when creating symlinks

Symptom:

- Install step fails with filesystem permission errors.

Fix:

- Check write access to install targets:
  - `~/.codex/skills/`
  - `~/.claude/skills/`
- Ensure parent directories exist and are writable.

## Unexpected root directory used

Root resolution order is:

1. `--root`
2. `LLM_SKILLS_ROOT`
3. `~/.llm/config.toml`
4. `~/.llm`

If behavior looks wrong, print your env and command flags, then retry with explicit `--root`.

## Validate code changes after edits

```bash
cargo fmt
cargo test
cargo check
```
