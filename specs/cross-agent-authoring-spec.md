# Cross-Agent Authoring Spec

## Purpose

Define one canonical skill source format and map it to Codex and Claude install targets.

## Canonical source of truth

- Root directory: configurable (default `~/.llm`)
- Skill source: `<root>/skills/<skill-name>/`
- Canonical required file: `SKILL.md`

## Canonical `SKILL.md`

`SKILL.md` must include YAML frontmatter:

- `name`: lowercase hyphen-case skill identifier
- `description`: trigger-oriented description of what the skill does and when to use it

Body is free-form markdown and may reference optional resources.

## Canonical resource directories

Optional:

- `scripts/`
- `references/`
- `assets/`

## Create-time intermediate brief

During `skillz create`, the selected agent writes `skill-brief.json` into the active work directory.

Expected schema:

```json
{
  "name": "example-skill",
  "description": "...",
  "overview": "...",
  "sections": [{ "title": "...", "body": "..." }],
  "scripts": [{ "path": "tool.py", "content": "..." }],
  "references": [{ "path": "guide.md", "content": "..." }],
  "assets": [{ "path": "template.txt", "content": "..." }]
}
```

## Render mapping

- Canonical source -> `<root>/renders/codex/<alias>/`
- Canonical source -> `<root>/renders/claude/<alias>/`

v1 mapping is a direct copy for both adapters.

## Install mapping

- Codex symlink: `<codex-root>/skills/<alias>` -> `<root>/renders/codex/<alias>`
- Claude symlink: `~/.claude/skills/<alias>` -> `<root>/renders/claude/<alias>`

## State model

`<root>/state/installs.json` stores alias install records for lookup, sync, and removal.

## Update behavior

`skillz update` edits canonical source first, then regenerates all installed renders and refreshes alias links.

## Delete behavior

`skillz delete` removes canonical source and all render/install aliases tied to the selected skill.
