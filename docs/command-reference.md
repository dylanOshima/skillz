# Command Reference

## Global options

- `--root <path>`: override the skill root directory for this run.
- `--agent codex|claude`: choose agent for create/update.
- `--yes`: skip confirmations where supported.

## `skillz create`

Create a new skill from an interactive deep-dive session.

Examples:

```bash
cargo run -- create
cargo run -- create --name api-checker --intent "Validate OpenAPI contracts" --agent codex
cargo run -- create --name pr-writer --install both --alias pr-helper
```

Behavior summary:

1. Collect name + intent.
2. Run interactive agent session.
3. Read generated brief JSON.
4. Generate canonical `SKILL.md` and optional resources.
5. Ask install target and alias.

## `skillz update`

Update an existing canonical skill.

Examples:

```bash
cargo run -- update
cargo run -- update --name api-checker --request "Add rollback section" --agent claude
```

Behavior summary:

1. Select skill.
2. Capture update request.
3. Run interactive session inside the skill folder.
4. Validate source.
5. Re-render and re-sync existing aliases.

## `skillz delete`

Delete a skill from source and aliases.

Examples:

```bash
cargo run -- delete
cargo run -- delete --name api-checker --yes
```

## `skillz list`

List canonical skills and installation paths.

Examples:

```bash
cargo run -- list
cargo run -- --root /tmp/my-llm-root list
```

## Case-insensitive aliases

All command names also work as title-case aliases:

- `Create`
- `Update`
- `Delete`
- `List`
