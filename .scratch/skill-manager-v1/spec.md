# Spec: skillmanager v1 — every agent's skills, one list, one file per skill

Status: ready-for-agent

## Problem Statement

I use four agents — Claude Code, Codex, Cursor, and omp — and each loads skills from its own skills folder. The same skills exist as separate real copies in several of those folders (about thirty skills are duplicated between Claude Code and omp alone; the `antislop-*` family is duplicated between Codex and Cursor). There is no single place where I can see every skill I own, which agents can load it, or which copies have drifted apart. Editing a skill means editing every copy by hand or letting them diverge; uninstalling means hunting folders; giving a new agent a skill means copying it and adding to the drift. Plugin and built-in skills are invisible entirely.

## Solution

A macOS desktop app (Tauri) that shows every skill from every agent in one list, marks which agents load each one, and makes one physical file serve all of them: linking a skill into another agent creates a symlink to the skill's master copy, so edits, renames, and uninstalls apply everywhere by construction. The app derives all state from disk on every launch (no database), never touches the filesystem without an explicit click, moves uninstalled skills to the macOS Trash so nothing is lost, and refuses to overwrite a divergent copy without showing a diff first. Plugin and built-in skills are listed read-only. The list updates live as folders change on disk.

## User Stories

1. As a skill owner, I want every skill from all my agents in one list, so that I can see what exists without hunting through four home-dotfolders.
2. As a skill owner, I want a row of agent icons on each skill, so that I can see at a glance which agents load it.
3. As a skill owner, I want each skill's description shown in the list, so that I can tell skills apart without opening them.
4. As a skill owner, I want newly created skills to come first by default, so that the freshest work is on top.
5. As a skill owner, I want to sort by name instead, so that I can find things alphabetically.
6. As a skill owner, I want to search by name or description with ⌘F, so that I can jump to a skill without scrolling.
7. As a skill owner, I want a "Missing" view listing skills that at least one of my agents can't see, so that gaps are obvious.
8. As a skill owner, I want Missing computed against all configured roots (including extra roots), so that "missing" is honest about everything an agent could see.
9. As a skill owner, I want a per-skill "link everywhere" action from the Missing view, so that closing a gap is one click.
10. As a skill owner, I want a "Duplicates" view pairing text-similar skills, so that I can find drift without AI.
11. As a skill owner, I want every copy of a skill listed with its location and whether it is a link or a real copy, so that I can audit exactly what's on disk.
12. As a skill owner, I want a warning when two copies of a skill have different content, so that edits don't silently diverge.
13. As a skill owner, I want a side-by-side diff of divergent copies, so that I can decide what to keep.
14. As a skill owner, I want "Add to" to link a skill folder into another agent's skills folder, so that every agent loads the same file.
15. As a skill owner, I want links to be symlinks to the master copy, so that an edit in one place is an edit everywhere.
16. As a skill owner, when the target agent already has a divergent copy, I want the app to refuse and show the diff, so that nothing is overwritten behind my back.
17. As a skill owner, I want an explicit "replace the target copy" choice that trashes the old copy first, so that a replacement is always recoverable.
18. As a skill owner, when a skill exists unlinked in two or more agents, I want the app to ask which copy becomes the master, so that the choice is mine.
19. As a skill owner, I want the master-selection prompt to default to a sensible agent order, so that the common case is one keystroke.
20. As a skill owner, I want to edit a skill's SKILL.md right in the app, so that I never open an editor for it.
21. As a skill owner, I want an edit to apply to the master and to every unlinked copy that has the same text, so that identical copies stay identical.
22. As a skill owner, I want an edit blocked behind the diff flow when a copy is divergent, so that I resolve the conflict before saving.
23. As a skill owner, I want to rename a skill once and have its folder, its links, and its SKILL.md name all update, so that renaming isn't four manual steps.
24. As a skill owner, I want renames recorded in a journal, so that my future usage history survives a rename.
25. As a skill owner, I want "Uninstall" to offer everywhere (default) or a single agent, so that I can remove a skill from one tool without losing it.
26. As a skill owner, I want uninstall to remove the links first and then move the master copy to the macOS Trash, so that nothing is silently deleted and everything is recoverable.
27. As a skill owner, I want an uninstall confirmation that lists exactly what will be unlinked and trashed, so that I can't destroy something by accident.
28. As a skill owner, I want plugin and built-in skills listed with their source, so that I can see everything an agent loads.
29. As a skill owner, I want a toolbar filter for my skills / plugin skills / built-in skills / all, so that the list separates what I manage from what agents manage.
30. As a skill owner, I want plugin and built-in skills to have no link/edit/rename/uninstall actions, so that the app never fights an agent's own updater.
31. As a skill owner, I want the list, the views, and the counts to update live as skills and chats change on disk, so that the app is never stale.
32. As a skill owner, I want the app to open read-only and only act on my explicit clicks, so that installing it can never reorganize my folders by itself.
33. As a skill owner, I want the app to launch fast with a full scan of all roots, so that I'm never waiting on a spinner for a hundred skills.
34. As a skill owner, I want extra roots (like a second Codex skills folder or an externally synced Cursor folder) addable through a plain config file, so that odd setups need no code change.
35. As a skill owner, I want the app to follow macOS light and dark appearance, so that it looks native without a settings trip.
36. As a skill owner, I want a helpful empty state when a view has nothing to show, so that "no duplicates" reads as good news, not as a bug.
37. As the developer, I want all filesystem logic in one testable Rust core with the UI as a thin layer over it, so that behavior is provable without GUI tests.
38. As the developer, I want a manual pre-release check that each agent actually loads a symlinked skill, so that the sharing model is proven before I rely on it.
39. As the app's only user, I want no code signing, no notarization, and no updater in v1, so that I can drag the app into /Applications and get on with it.


- Greenfield build: Tauri 2 desktop app, single window. Frontend: React with Vite + TypeScript + Tailwind, with shadcn/ui as the component library (copied-in components on Radix primitives). Backend: Rust only, no Node sidecar.
- Architecture in three layers: a **Rust core crate** (all scanning, comparison, linking, renaming, editing, uninstall, watching logic), a **Tauri command layer** that wraps the core's public API one-to-one and serializes results as JSON, and the **React UI** as a thin view over those commands. The UI holds no filesystem logic.
- Agents and roots: the four canonical roots (Claude Code, Codex, Cursor, omp personal skills folders) are hardcoded; extra roots come from a plain JSON config file owned by the app. Plugin and built-in skill sources are scanned for display only.
- Identity: a skill is its folder name. Frontmatter `name` is display-only and follows renames. Grouping across roots is by folder name.
- Statelessness: every launch re-scans all roots and derives the full state; nothing is persisted except the roots config and the rename journal (old name, new name, timestamp) — both deletable without harming any agent.
- Master copy and linking: "Add to" (and "link everywhere") creates a symlink in the target root pointing at the master copy. When unlinked duplicates exist, the user picks the master (default: canonical-root order). All non-master copies become links.
- Divergence guard: linking, editing, or replacing onto a divergent copy is refused until the user resolves a side-by-side diff; "replace" moves the old copy to the Trash before the link is created.
- Edit: writes the master copy and every unlinked copy whose text is identical; divergent copies trigger the diff flow before saving.
- Rename: renames the master folder, relinks every link, updates the SKILL.md frontmatter name, and appends to the rename journal.
- Uninstall: removes all links first, then moves the master to the macOS Trash via a Trash-aware library. Scope is per-agent or everywhere (default everywhere), chosen in a confirmation that lists every affected path. Plugin and built-in skills are read-only.
- Missing: computed per skill against the full set of configured roots.
- Duplicates: text-similarity pairing (normalized content comparison); no AI in v1.
- Live updates: filesystem watcher (debounced) triggers a rescan and pushes refreshed state to the UI.
- Sorting: creation date (directory birth time) by default, then name; both directions. Usage-based sorts arrive with v1.5.
- Search: ⌘F over skill name and description.
- Appearance: follows the macOS system theme; no in-app toggle.
- Distribution: personal, unsigned build; drag-install. No updater, no release pipeline.

## Testing Decisions

- A good test exercises external behavior at the single seam — the Rust core crate's public API: given a fixture roots tree on disk, scanning returns the expected derived state, and operations produce the expected filesystem outcome. Tests assert on the filesystem (link targets, folder names, file contents, presence in a Trash fixture) and on returned state, never on internals.
- The core is tested with integration tests against temp-dir fixtures that mimic real agent roots: folders with real SKILL.md files, frontmatter variety (missing name, differing descriptions), pre-existing symlinks, divergent copies, and mixed canonical/extra roots.
- Operation tests cover the guards: link onto divergent copy refuses; replace trashes first; edit writes all identical copies only; uninstall removes links before trashing; rename updates folder, links, and frontmatter and journals the rename.
- Watcher behavior is tested at the core level by emitting fixture file events and asserting the rescan result (not timing-sensitive assertions).
- The Tauri command layer is a thin wrapper and is not separately tested. The React UI is not tested.
- Prior art: none — this is the repo's first code; these tests become the prior art for future specs.
- The one thing automated tests cannot prove — that each agent actually loads a symlinked skill — is a documented manual pre-release check per agent (Claude Code, Codex, Cursor, omp).

## Out of Scope

- Usage statistics, the Unused view, and usage-based sorting (v1.5, separate spec: Claude Code and omp explicit invocation events only).
- All AI features: Explain with AI, Merge with AI, AI-powered similar-skills, Suggestions (v2).
- The `skillscout` terminal command (v2).
- GitHub auto-update, code signing, notarization, any release pipeline.
- Codex and Cursor usage detection (until their transcripts provably record invocations).
- Central skill store or any migration of skills into app-owned locations.
- Auto-dedupe or any filesystem mutation without an explicit user action.
- Plugin skill installation/updating; managing agents' plugin systems.
- Windows and Linux.
- UI-level automated tests.

## Further Notes

- Design was grilled to completion before this spec; the settled decision log lives in ADR-0001 (stateless, symlinks) and the glossary.
- The single risk the design rests on: an agent refusing to load symlinked skills. The manual per-agent check is the release gate; if an agent fails it, the fallback for that agent is copy-on-edit (out of scope until needed).
- Process constraint for implementing agents: repo owner requires explicit approval before any commit, frontend production build, or dev-server run. Code edits, typecheck, lint, and unit/integration tests run freely; end-to-end smoke requires one approved `dev server` command.
- Real-world fixtures for manual testing already exist on this machine: duplicated skills between Claude Code and omp, and `antislop-*` between Codex and Cursor.
