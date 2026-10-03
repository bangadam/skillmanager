# 03 — Extra roots, plugin & built-in skills, filter toolbar

**What to build:** Everything an agent can load becomes visible. Extra roots (a second Codex skills folder, an externally synced Cursor folder) join the scan via a plain JSON config file the app owns. Plugin and built-in skill sources are scanned and listed read-only with their source marked. A toolbar filter switches between: my skills / plugin skills / built-in skills / all.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** resolved

- [x] Adding a path to the config file makes its skills appear in the list on next launch (and next rescan).
- [x] Plugin and built-in skills appear with their source agent marked.
- [x] The toolbar filter: mine / plugin / built-in / all — each works and shows its count.
- [x] Plugin and built-in skills expose no link, edit, rename, or uninstall actions.
- [x] Plugin and built-in skills are never counted as Missing (they're managed by their agent, not by roots).
- [x] A malformed or missing config file is reported in the UI, never silently ignored.

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit d2873fa. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
