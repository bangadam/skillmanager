# skillmanager

macOS desktop app (Tauri 2 + React) that unifies skill management across agents (Claude Code, Codex, Cursor, omp). See `.scratch/skill-manager-v1/spec.md` for the v1 spec.

- Backend logic lives in one Rust core crate; Tauri commands wrap it 1:1; the React UI is a thin view.
- All tests target the Rust core API against temp-dir fixtures. No UI tests, no command-layer tests.
- Disk is the source of truth: no database, no central store (ADR-0001).

## Agent skills

### Issue tracker

Issues and specs live as local markdown under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default mattpocock vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
