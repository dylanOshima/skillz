# Codex Skills Spec (Project Reference)

## Skill location

Codex skills are installed as directories under:

- `$CODEX_HOME/skills/` when `CODEX_HOME` is set
- Otherwise `~/.codex/skills/`

Each installed skill is represented as a symlink alias created by `skillz`.

## Required file

- `SKILL.md`

`SKILL.md` should include YAML frontmatter with:

- `name`
- `description`

## Optional files

- `scripts/`
- `references/`
- `assets/`
- `agents/openai.yaml`

## Invocation model

Codex can invoke skills explicitly and through trigger descriptions in frontmatter metadata. `skillz` keeps canonical source in `<root>/skills/` and renders installable copies to `<root>/renders/codex/`.

## Rendering policy in v1

- Render is currently a direct copy of canonical source.
- Future versions can add codex-specific transforms (for example, generating `agents/openai.yaml` when missing).
