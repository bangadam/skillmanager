# 01 — Tracer bullet: app shell and the skill list

**What to build:** The app exists and tells the truth about disk. Launching it opens a single macOS window listing every skill from the four canonical roots — name, description, and a row of agent icons marking which agents load it. All scanning lives in the Rust core; the UI is a thin view over one Tauri command. Scan behavior is pinned by integration tests against temp-dir fixtures from day one. The window follows macOS light and dark.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] App launches to a single window with zero manual configuration.
- [ ] Every skill in the four canonical roots appears once, grouped by folder name (identity), showing name and description from SKILL.md frontmatter.
- [ ] Agent icons reflect which roots currently hold the skill (real copy or link).
- [ ] The listed skill count matches disk reality.
- [ ] Frontmatter quirks (missing name, missing description) degrade gracefully; the scan never fails on a malformed SKILL.md.
- [ ] Rust core scan is covered by integration tests over temp-dir fixtures: frontmatter variety, empty roots, nested subfolders.
- [ ] Window appearance follows the macOS system theme.
- [ ] shadcn/ui is initialized and the list is built from its primitives.
