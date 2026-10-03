# 02 — Make the list navigable: search, sort, counts

**What to build:** The list becomes fast to move around in: ⌘F focuses a search box filtering by skill name or description; the list sorts by creation date (newest first, the default) or by name, both directions; every view shows a live count; empty views show a friendly empty state.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** ready-for-agent

- [ ] ⌘F focuses search; typing filters skills whose name or description matches, case-insensitively.
- [ ] Clearing search restores the full list.
- [ ] Default sort: newest-created first, using directory birth time.
- [ ] Sort by name ascending/descending; creation date both directions; choice persists for the session.
- [ ] The visible count updates with every filter/sort/view change.
- [ ] Empty results render an explanatory empty state, not a blank window.
