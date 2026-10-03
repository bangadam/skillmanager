# ADR-0001: Stateless disk-as-truth with symlink sharing

Date: 2026-10-03
Status: accepted

## Context

Skills already exist as duplicated real copies across the four agents' skills folders on this laptop (~30 skills duplicated between Claude Code and omp alone; `antislop-*` between Codex and Cursor). The app must let one edit, rename, or uninstall apply everywhere without becoming another sync tool that owns data and drifts.

## Decision

The app keeps no database and no central skill store. All state is re-derived from disk on every launch.

- Skill identity = folder name; frontmatter `name` is display-only.
- One copy of a skill is the **master copy**; every other copy becomes a **link** (symlink) to it. Edits and renames propagate by construction of the filesystem, not by app logic.
- The only persistent app-owned artifacts are the roots config file and the rename journal (old name → new name), both plain files, both deletable without breaking agents.
- The app never mutates the filesystem without an explicit user action (no auto-dedupe).

## Consequences

- No migration step; deleting the app leaves agents fully functional.
- A master-selection rule is required when unlinked duplicates exist (user picks; default: canonical-root order).
- The whole model rests on agents following symlinks — verified manually per agent before release.
- Cross-rename usage continuity (v1.5) requires the rename journal, since transcripts record skill names, not paths.
