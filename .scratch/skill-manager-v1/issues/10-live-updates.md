# 10 — Live updates while you work

**What to build:** The app never goes stale. A filesystem watcher on all configured roots triggers a debounced rescan, and the refreshed state is pushed to the UI: skills created, edited, renamed, or deleted in a terminal or by an agent appear in the list and the counts without relaunching.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** resolved

- [x] Creating a skill folder in any root from outside the app appears in the list without relaunch, within a couple of seconds.
- [x] Editing a SKILL.md externally updates the skill's description; deleting the folder removes the skill.
- [x] Counts in every view update with the rescan.
- [x] Rapid multi-file changes coalesce into one rescan (debounced), without UI thrash.
- [x] The watcher survives an error on a watched path (deleted root, permission change) without crashing.
- [x] The current selection and scroll position survive a rescan where the underlying skill still exists.
- [x] Watcher behavior is tested at the core seam by emitting fixture file events and asserting rescan results.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit 48e438f. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
