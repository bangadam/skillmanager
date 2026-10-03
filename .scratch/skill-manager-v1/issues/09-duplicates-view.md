# 09 — Duplicates view: text-similar pairing

**What to build:** A read-only radar for drift. The Duplicates view pairs skills whose SKILL.md reads alike (normalized text comparison, no AI), and for each skill shows every copy with its location and whether it's a real copy or a link. Exact duplicates are included; the view is the non-AI predecessor of the v2 similar-skills feature.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** resolved

- [x] Skills with near-identical or identical SKILL.md content are paired and listed.
- [x] Every copy of a paired skill shows its root and a real-copy vs link marker.
- [x] Exact duplicates (same normalized content) are always paired.
- [x] Comparison is deterministic: same disk state yields the same pairing.
- [x] No AI, no network, no external process.
- [x] Selecting a pair offers a path into the linking flow once ticket 04 lands (graceful absence before that).

## Comments

- Resolved on integration/skill-manager-v1 (implement-spec run, 2026-10-04). Landed in commit cd387dc. Verified: cargo test --workspace 51 passed, cargo clippy --workspace --all-targets -- -D warnings clean, npx tsc --noEmit clean. Visual/dev-server check deferred per repo owner's command policy.
