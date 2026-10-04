# Glossary

Vocabulary for skillmanager. Use these terms in issues, specs, and tests.

- **Agent** — a CLI/editor that loads skills from its own skills folder. v1 agents: Claude Code, Codex, Cursor, omp, Hermes.
- **Skill** — a folder containing a `SKILL.md` that an agent can load. Identified by its folder name.
- **Root** — a skills folder an agent reads. **Canonical root**: one of the five hardcoded agent skills folders. **Extra root**: any additional folder added through the config file (e.g. a second Codex skills dir). Flat agents load `root/<skill>`; Hermes nests skills under group folders (`root/<group>/<skill>`, any depth) that carry no `SKILL.md`.
- **Copy** — a skill's physical instance in one root.
- **Master copy** — the one real directory that a skill's other copies link to.
- **Link** — a symlink in a root pointing at the master copy.
- **Unlinked duplicate** — a skill existing as real (non-link) copies in two or more roots.
- **Divergent copies** — unlinked copies of the same skill whose contents differ.
- **Missing** — a skill visible to at least one agent but absent from at least one configured root.
- **Duplicates view** — the pairing of text-similar skills (no AI in v1).
- **Plugin skill / built-in skill** — skills managed by an agent's own plugin or system machinery. Never linked, edited, or renamed by the app; may be uninstalled (trashes the agent's cached copy, which a plugin re-sync may restore).
- **Scan** — reading all roots to derive the full state. State is always derived from disk, never persisted.
- **Rename journal** — the app-owned `~/.config/skillmanager/renames.json` array of `{from, to, at}` entries (`at` is Unix seconds), kept so future usage attribution survives renames. Missing means empty history.
- **Usage event** (v1.5) — an explicit skill invocation recorded in a chat transcript (Claude Code `Skill` tool events; omp `skill://` reads). Advertised catalogs and prompt mentions never count.
- **Collection** — the sidebar grouping of skills by source: your own skills, one named plugin (e.g. `mattpocock`), or one agent's built-in/plugin set. The unit of bulk removal: plan once, confirm, trash every member.
- **Category** — a skill's coarse topic: `security`, `data`, `ai`, `devops`, `research`, `coding`, `writing`, `design`, or `other`. Derived at scan time: the `category:` frontmatter field wins when present, otherwise inferred from the skill's name and description by whole-word keyword hit count. First copy parsed wins, like name and description.
