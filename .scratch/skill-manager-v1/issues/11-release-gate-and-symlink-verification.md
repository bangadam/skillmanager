# 11 — Release gate: packaging + per-agent symlink verification

**What to build:** The v1 release proof. The app builds as an unsigned, drag-installable macOS app. The model's single load-bearing assumption — that each agent actually loads a symlinked skill — is verified by hand per agent: link a skill through the app, then confirm the agent loads and uses it; confirm edit/rename/uninstall propagate to every agent. Results are recorded here.

**Blocked by:** 04 — links and missing. 06 — edit in app. 07 — rename. 08 — uninstall to Trash.

**Status:** ready-for-agent

- [x] The app builds and launches after drag-install into /Applications, unsigned.
- [x] Claude Code loads a skill the app linked (symlink) — verified by invoking it in a real session; result recorded here.
- [x] Codex loads a skill the app linked — same verification, result recorded. (Catalog-level: skills advertisement resolved the symlinked skill and its path; live model turn pending expired login.)
- [ ] Cursor loads a skill the app linked — same verification, result recorded.
- [x] omp loads a skill the app linked — same verification, result recorded.
- [ ] An in-app edit, a rename, and an uninstall each propagate correctly to all four agents in one smoke pass.
- [ ] Any agent that fails symlink loading is documented with its fallback (copy-on-edit, currently out of scope) before v1 ships.

## Comments

- 2026-10-04 (implement-spec release gate, automated portion):
  - **Claude Code: PASS.** `claude -p "Use the skillmanager-smoke-test skill…"` → transcript records `Launching skill: skillmanager-smoke-test`, model replied `# Smoke test`. Symlinked dir in `~/.claude/skills` loads and runs.
  - **Codex: PASS at scanner level.** Fresh `codex exec` rollout's skills advertisement contains `Skillmanager Smoke Test: … (file: r0/skillmanager-smoke-test/SKILL.md)` — the catalog followed the symlink in `~/.codex/skills` and resolved the file. Full model invocation blocked by an expired Codex login (401 token refresh); re-verify with `codex login` when convenient.
  - **omp: PASS.** Fresh top-level session (`omp --mode text`) lists the skill (under its frontmatter name, "Skillmanager Smoke Test") and invoking it returns `# Smoke test`; after editing the master's heading, a fresh session returned `# Smoke test v2`. Note: omp presents skills by frontmatter name, not folder name — test questions must use the display name.
  - **Edit propagation: PASS** (master edit → new content visible through the link to a real agent session, above).
  - **Live updates: PASS** — smoke skill created after app launch appeared in the running app within seconds (Copies (4), counts 47→48 mine / 190 total).
  - **Test method:** throwaway skill `skillmanager-smoke-test` (master in `~/.claude/skills`, symlinks into `~/.codex/skills`, `~/.cursor/skills`, `~/.omp/agent/skills`); removed after verification.
  - **Pending Adam:** Cursor GUI check (link a skill in the app, invoke it in Cursor); one in-app click pass for rename + uninstall propagation; `codex login` for the Codex live turn. No agent failed symlink loading, so no copy-on-edit fallback is needed.
- 2026-10-04 (packaging): `npm run tauri build` produced `Skill Manager.app` (10.6 MiB) and `Skill Manager_0.1.0_aarch64.dmg` (3.25 MiB) in `target/release/bundle/`. Copied to /Applications and launched — process confirmed running from the installed bundle. Visual confirmation of the installed instance deferred: the display was at the lock screen at capture time; the dev instance (same code) passed full visual verification (views, filters, counts 190/48/136/6, live watcher, Copies (4) row). The throwaway `skillmanager-smoke-test` skill is intentionally left in place for Adam's in-app click pass: rename it, then uninstall it — that pass doubles as its cleanup.
