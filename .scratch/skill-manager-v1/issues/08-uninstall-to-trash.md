# 08 — Uninstall to the macOS Trash

**What to build:** Removal that's always recoverable. Uninstall opens a confirmation listing every path that will be affected; the scope is one agent or everywhere (default). Links are removed first, then the master copy moves to the macOS Trash. If the scope removes the root holding the master, another copy is promoted to master so the remaining agents keep working. Plugin and built-in skills can't be uninstalled.

**Blocked by:** 04 — links: Missing view, Add to, link everywhere.

**Status:** resolved

- [x] The confirmation dialog lists every link and the master copy that will be affected, before anything happens.
- [x] "Everywhere" (default): all links removed, then the master copy lands in the macOS Trash.
- [x] Single-agent scope: only that root's link is removed; the skill keeps working for every other agent.
- [x] Single-agent scope on the root holding the master: another copy is promoted to a real directory and remaining links re-point to it; if it was the only copy, the dialog guides to "everywhere".
- [x] The trashed skill is recoverable from the macOS Trash.
- [x] Plugin and built-in skills offer no uninstall action.
- [x] Links are always removed before the master is trashed (integration test asserts order).

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit 0c167e3. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
