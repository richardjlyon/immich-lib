# Phase 06-04: Docker Test Environment - Context

**Gathered:** 2025-12-27
**Status:** Ready for planning

<vision>
## How This Should Work

A Docker environment that spins up a clean Immich instance, seeds the synthetic fixtures, and lets us verify that Immich detects them as duplicates.

Start with the simplest approach: clean bootstrap from scratch each time. No snapshots, no state management. If Immich initialization takes 30-60 seconds, that's acceptable — simplicity matters more than speed at this stage.

The key validation is that Immich actually recognizes the synthetic fixture pairs as duplicates. Without that, the whole testing approach fails. This environment validates the *algorithm logic* with controlled scenarios; real-world messiness gets validated in Phase 7 with a cloned instance.

</vision>

<essential>
## What Must Be Nailed

- **Reliable duplicate detection** — Immich must consistently recognize the synthetic fixtures as duplicate pairs. This is the entire point of the phase.
- **Clean bootstrap works** — Start from scratch each time, no leftover state from previous runs
- **Fixtures get seeded correctly** — The 34 scenarios from 06-03 upload and are processed by Immich

</essential>

<boundaries>
## What's Out of Scope

- CI/GitHub Actions automation — keep it local-only for now
- Performance optimization — don't worry about startup time, make it work first
- Real library data — no real photos in this phase, that's Phase 7
- Snapshot/reset complexity — only add if clean bootstrap proves too slow

</boundaries>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches for the interface (shell scripts, make targets, whatever works cleanly).

</specifics>

<notes>
## Additional Context

User's core concern: synthetic tests might appear to work but fail at scale with 2000+ real images. The mitigation is the two-layer testing strategy:
1. Phase 6 (synthetic) validates algorithm logic with controlled edge cases
2. Phase 7 (clone) validates real-world behavior with actual messy data

Visual confirmation of duplicate detection in Immich UI is desired, but user needs guidance on what to look for. The fear is false confidence from tests that pass but don't catch real-world edge cases.

</notes>

---

*Phase: 06-synthetic-integration-tests*
*Context gathered: 2025-12-27*
