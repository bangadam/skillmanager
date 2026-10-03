# 02 — Make the list navigable: search, sort, counts

**What to build:** The list becomes fast to move around in: ⌘F focuses a search box filtering by skill name or description; the list sorts by creation date (newest first, the default) or by name, both directions; every view shows a live count; empty views show a friendly empty state.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** resolved

- [x] ⌘F focuses search; typing filters skills whose name or description matches, case-insensitively.
- [x] Clearing search restores the full list.
- [x] Default sort: newest-created first, using directory birth time.
- [x] Sort by name ascending/descending; creation date both directions; choice persists for the session.
- [x] The visible count updates with every filter/sort/view change.
- [x] Empty results render an explanatory empty state, not a blank window.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit 6796fe0. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
