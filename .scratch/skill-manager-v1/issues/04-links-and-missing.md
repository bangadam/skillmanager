# 04 — Links: Missing view, Add to, link everywhere

**What to build:** The dedupe heart. A Missing view lists every skill that at least one configured root can't see, computed against the full set of configured roots. "Add to" links a skill into another agent's root as a symlink to its master copy; "link everywhere" closes every gap at once. When a skill exists as unlinked duplicates, the app asks which copy becomes the master (default: canonical-root order) before any link is created. Linking onto a target that already holds a copy is refused with a clear error — the diff flow lives in ticket 05.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** resolved

- [x] Missing view lists skills absent from at least one configured root, and only those.
- [x] "Add to" creates a symlink in the chosen root pointing at the skill's master copy.
- [x] "Link everywhere" creates links in every absent root in one action.
- [x] After linking, the skill's icon row gains the target agent; the target root holds a symlink.
- [x] An edit to the master copy is visible through every link (integration test).
- [x] Linking a skill with unlinked duplicates prompts for the master copy, defaulting to canonical-root order.
- [x] After master selection, non-master copies become links to the master.
- [x] Linking onto a target that already holds any copy is refused with an actionable error.
- [x] No filesystem change ever happens without an explicit click.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit e783961. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
