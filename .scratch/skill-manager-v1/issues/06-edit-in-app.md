# 06 — Edit SKILL.md in the app

**What to build:** Editing without leaving the app. Opening a skill's SKILL.md shows an editor with its current text; saving writes the master copy and every unlinked copy that had identical text, so identical copies stay identical. A divergent copy blocks the save until the diff flow from ticket 05 resolves it. Links need no extra writes — they see the master by construction.

**Blocked by:** 04 — links: Missing view, Add to, link everywhere. 05 — divergence guard: warning, diff, replace.

**Status:** ready-for-agent

- [ ] The editor opens with the skill's current SKILL.md text.
- [ ] Saving writes the master copy; every link reflects the change (integration test).
- [ ] Identical unlinked copies receive the same write and stay identical afterwards.
- [ ] A divergent copy blocks saving until resolved through the diff/replace flow.
- [ ] The save result reports which files were written.
- [ ] A skill with no SKILL.md can be given one through the editor.
