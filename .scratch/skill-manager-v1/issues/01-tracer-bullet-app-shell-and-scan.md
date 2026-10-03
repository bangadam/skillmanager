# 01 — Tracer bullet: app shell and the skill list

**What to build:** The app exists and tells the truth about disk. Launching it opens a single macOS window listing every skill from the four canonical roots — name, description, and a row of agent icons marking which agents load it. All scanning lives in the Rust core; the UI is a thin view over one Tauri command. Scan behavior is pinned by integration tests against temp-dir fixtures from day one. The window follows macOS light and dark.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] App launches to a single window with zero manual configuration.
- [x] Every skill in the four canonical roots appears once, grouped by folder name (identity), showing name and description from SKILL.md frontmatter.
- [x] Agent icons reflect which roots currently hold the skill (real copy or link).
- [x] The listed skill count matches disk reality.
- [x] Frontmatter quirks (missing name, missing description) degrade gracefully; the scan never fails on a malformed SKILL.md.
- [x] Rust core scan is covered by integration tests over temp-dir fixtures: frontmatter variety, empty roots, nested subfolders.
- [x] Window appearance follows the macOS system theme.
- [x] shadcn/ui is initialized and the list is built from its primitives.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit 3603838. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
