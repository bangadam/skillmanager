# 09 — Duplicates view: text-similar pairing

**What to build:** A read-only radar for drift. The Duplicates view pairs skills whose SKILL.md reads alike (normalized text comparison, no AI), and for each skill shows every copy with its location and whether it's a real copy or a link. Exact duplicates are included; the view is the non-AI predecessor of the v2 similar-skills feature.

**Blocked by:** 01 — tracer bullet: app shell and the skill list.

**Status:** ready-for-agent

- [ ] Skills with near-identical or identical SKILL.md content are paired and listed.
- [ ] Every copy of a paired skill shows its root and a real-copy vs link marker.
- [ ] Exact duplicates (same normalized content) are always paired.
- [ ] Comparison is deterministic: same disk state yields the same pairing.
- [ ] No AI, no network, no external process.
- [ ] Selecting a pair offers a path into the linking flow once ticket 04 lands (graceful absence before that).
