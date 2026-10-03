# 10 — Live updates while you work

**What to build:** The app never goes stale. A filesystem watcher on all configured roots triggers a debounced rescan, and the refreshed state is pushed to the UI: skills created, edited, renamed, or deleted in a terminal or by an agent appear in the list and the counts without relaunching.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** ready-for-agent

- [ ] Creating a skill folder in any root from outside the app appears in the list without relaunch, within a couple of seconds.
- [ ] Editing a SKILL.md externally updates the skill's description; deleting the folder removes the skill.
- [ ] Counts in every view update with the rescan.
- [ ] Rapid multi-file changes coalesce into one rescan (debounced), without UI thrash.
- [ ] The watcher survives an error on a watched path (deleted root, permission change) without crashing.
- [ ] The current selection and scroll position survive a rescan where the underlying skill still exists.
- [ ] Watcher behavior is tested at the core seam by emitting fixture file events and asserting rescan results.
