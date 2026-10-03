# 11 — Release gate: packaging + per-agent symlink verification

**What to build:** The v1 release proof. The app builds as an unsigned, drag-installable macOS app. The model's single load-bearing assumption — that each agent actually loads a symlinked skill — is verified by hand per agent: link a skill through the app, then confirm the agent loads and uses it; confirm edit/rename/uninstall propagate to every agent. Results are recorded here.

**Blocked by:** 04 — links and missing. 06 — edit in app. 07 — rename. 08 — uninstall to Trash.

**Status:** ready-for-agent

- [ ] The app builds and launches after drag-install into /Applications, unsigned.
- [ ] Claude Code loads a skill the app linked (symlink) — verified by invoking it in a real session; result recorded here.
- [ ] Codex loads a skill the app linked — same verification, result recorded.
- [ ] Cursor loads a skill the app linked — same verification, result recorded.
- [ ] omp loads a skill the app linked — same verification, result recorded.
- [ ] An in-app edit, a rename, and an uninstall each propagate correctly to all four agents in one smoke pass.
- [ ] Any agent that fails symlink loading is documented with its fallback (copy-on-edit, currently out of scope) before v1 ships.
