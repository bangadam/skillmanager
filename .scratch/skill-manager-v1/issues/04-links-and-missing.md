# 04 — Links: Missing view, Add to, link everywhere

**What to build:** The dedupe heart. A Missing view lists every skill that at least one configured root can't see, computed against the full set of configured roots. "Add to" links a skill into another agent's root as a symlink to its master copy; "link everywhere" closes every gap at once. When a skill exists as unlinked duplicates, the app asks which copy becomes the master (default: canonical-root order) before any link is created. Linking onto a target that already holds a copy is refused with a clear error — the diff flow lives in ticket 05.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** ready-for-agent

- [ ] Missing view lists skills absent from at least one configured root, and only those.
- [ ] "Add to" creates a symlink in the chosen root pointing at the skill's master copy.
- [ ] "Link everywhere" creates links in every absent root in one action.
- [ ] After linking, the skill's icon row gains the target agent; the target root holds a symlink.
- [ ] An edit to the master copy is visible through every link (integration test).
- [ ] Linking a skill with unlinked duplicates prompts for the master copy, defaulting to canonical-root order.
- [ ] After master selection, non-master copies become links to the master.
- [ ] Linking onto a target that already holds any copy is refused with an actionable error.
- [ ] No filesystem change ever happens without an explicit click.
