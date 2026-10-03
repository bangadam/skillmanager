# 05 — Divergence guard: warning, diff, replace

**What to build:** The safety net under every overwrite. Copies of the same skill are compared; when content differs, the skill is flagged and a side-by-side diff opens. An explicit "replace" choice moves the losing copy to the macOS Trash first, then links the winner — nothing is overwritten in place. This flow guards linking (04's refusal), and is reused by edit (06).

**Blocked by:** 04 — links: Missing view, Add to, link everywhere.

**Status:** resolved

- [x] A skill with divergent copies shows a visible warning in the list and in its copies list.
- [x] Identical copies never warn.
- [x] The diff view shows the two SKILL.md contents side by side, differences highlighted.
- [x] "Replace" requires an explicit confirmation naming exactly what gets trashed.
- [x] Replacing moves the old copy to the macOS Trash, then creates the link — verified in that order (integration test).
- [x] The replaced copy is recoverable from the Trash.
- [x] Comparison covers SKILL.md text; a missing SKILL.md in one copy counts as divergent.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit 23647b8. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
