# Phase 7: Live Instance Validation - Context

**Gathered:** 2025-12-27
**Updated:** 2025-12-27
**Status:** In progress (plan 07-01 complete)

<vision>
## How This Should Work

Run the full workflow (analyze → execute) against the test Immich database from Phase 6, then exhaustively verify the end state. The test database is small enough that we don't need sampling - we can check every surviving image.

This is the final proof that the tool works correctly before considering use on production data. We need to verify that:
1. Scoring picks the correct winners
2. Downloads create valid backups
3. Metadata consolidation actually transfers EXIF data in Immich's database
4. Deletion removes only the losers, leaving winners intact

The key insight: we unit-tested the consolidation logic, but never ran it against a real Immich instance to confirm the API calls work and metadata actually transfers. This phase closes that gap.

**Restore capability:** The tool needs a `restore` command that can re-upload backed-up files to Immich. This provides a tested escape hatch before running on production. If something goes wrong at scale, the tool itself can undo what it did rather than requiring complex NAS snapshot + VM backup restoration.

</vision>

<essential>
## What Must Be Nailed

- **Metadata transfer verification** - Prove consolidation actually works by running against real Immich database and checking EXIF survived
- **Full workflow execution** - Run analyze → execute on test database, not just unit tests
- **Complete end-state validation** - Every surviving image checked, not sampling
- **Restore command** - Re-upload backed-up losers to Immich as new assets

</essential>

<boundaries>
## What's Out of Scope

- Production database - this phase uses only the test instance from Phase 6
- Production run decision - proving the tool works is separate from deciding to run on production
- New test scenarios - validate with existing fixtures from Phase 6
- Full state restoration - restore creates new assets, not original IDs

</boundaries>

<specifics>
## Specific Ideas

- Use the Docker Immich environment from Phase 6 with existing fixtures
- The test database is small enough for exhaustive verification (no sampling needed)
- 06-01 test candidate finder output may inform what to look for
- Restore command re-uploads from backup directory created by execute

</specifics>

<notes>
## Additional Context

User has 2000+ real duplicates in production. The concern is ensuring zero data loss at that scale. This validation phase proves the tool is trustworthy before considering production use.

The gap identified: consolidation logic was unit-tested but never integration-tested against actual Immich API to confirm metadata transfer works end-to-end.

**Production environment:**
- Immich runs on an Unraid VM
- Images stored on NAS with snapshot capability
- Recovery complexity is the concern: coordinating NAS snapshot + VM restore is painful
- Having restore built into the tool provides simpler recovery path

</notes>

---

*Phase: 07-live-instance-validation*
*Context gathered: 2025-12-27*
