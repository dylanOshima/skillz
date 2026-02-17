# Claude Code Skills Spec (Project Reference)

## Skill location

Claude Code skills are installed as directories under:

- `~/.claude/skills/`

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

Claude-specific metadata can be added as needed in frontmatter, as long as shared fields remain valid.

## Invocation model

Claude Code can use skills from local/project skill directories. `skillz` manages shared canonical source and emits Claude installable copies in `<root>/renders/claude/`.

## Rendering policy in v1

- Render is currently a direct copy of canonical source.
- Future versions can add claude-specific validation or frontmatter extension handling.
