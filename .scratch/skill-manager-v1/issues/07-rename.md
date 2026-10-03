# 07 — Rename once: folder, links, SKILL.md, journal

**What to build:** Renaming a skill is one action: the master folder takes the new name, every link is re-created against the new path, the SKILL.md frontmatter name matches, and the rename is recorded in the rename journal so future usage attribution survives. Unlinked duplicate copies are untouched — link first, then rename.

**Blocked by:** 04 — links: Missing view, Add to, link everywhere.

**Status:** ready-for-agent

- [ ] Renaming renames the master folder on disk.
- [ ] Every link points at the renamed path afterwards (no dangling links; integration test).
- [ ] SKILL.md frontmatter `name` matches the new name.
- [ ] The rename journal records old name, new name, and timestamp.
- [ ] The old name leaves no residue: the skill groups under its new name, agent icons unchanged.
- [ ] Renaming a name that collides with an existing skill is refused.
- [ ] Unlinked duplicate copies are not renamed.
