# Glossary

Vocabulary for skillmanager. Use these terms in issues, specs, and tests.

- **Agent** — a CLI/editor that loads skills from its own skills folder. v1 agents: Claude Code, Codex, Cursor, omp.
- **Skill** — a folder containing a `SKILL.md` that an agent can load. Identified by its folder name.
- **Root** — a skills folder an agent reads. **Canonical root**: one of the four hardcoded agent skills folders. **Extra root**: any additional folder added through the config file (e.g. a second Codex skills dir).
- **Copy** — a skill's physical instance in one root.
- **Master copy** — the one real directory that a skill's other copies link to.
- **Link** — a symlink in a root pointing at the master copy.
- **Unlinked duplicate** — a skill existing as real (non-link) copies in two or more roots.
- **Divergent copies** — unlinked copies of the same skill whose contents differ.
- **Missing** — a skill visible to at least one agent but absent from at least one configured root.
- **Duplicates view** — the pairing of text-similar skills (no AI in v1).
- **Plugin skill / built-in skill** — skills managed by an agent's own plugin or system machinery. Displayed read-only; never linked, edited, renamed, or uninstalled by the app.
- **Scan** — reading all roots to derive the full state. State is always derived from disk, never persisted.
- **Rename journal** — the app-owned record of renames (old name → new name, timestamp), kept so that future usage attribution survives renames.
- **Usage event** (v1.5) — an explicit skill invocation recorded in a chat transcript (Claude Code `Skill` tool events; omp `skill://` reads). Advertised catalogs and prompt mentions never count.
