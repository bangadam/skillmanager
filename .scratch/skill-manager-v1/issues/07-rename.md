# 07 — Rename once: folder, links, SKILL.md, journal

**What to build:** Renaming a skill is one action: the master folder takes the new name, every link is re-created against the new path, the SKILL.md frontmatter name matches, and the rename is recorded in the rename journal so future usage attribution survives. Unlinked duplicate copies are untouched — link first, then rename.

**Blocked by:** 04 — links: Missing view, Add to, link everywhere.

**Status:** resolved

- [x] Renaming renames the master folder on disk.
- [x] Every link points at the renamed path afterwards (no dangling links; integration test).
- [x] SKILL.md frontmatter `name` matches the new name.
- [x] The rename journal records old name, new name, and timestamp.
- [x] The old name leaves no residue: the skill groups under its new name, agent icons unchanged.
- [x] Renaming a name that collides with an existing skill is refused.
- [x] Unlinked duplicate copies are not renamed.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit f9a9c0c. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
