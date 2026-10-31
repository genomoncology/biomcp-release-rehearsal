# BioASQ benchmark stays behind in 1.0

Decided 2026-10-31 by the PM, after Jarvis's 1.0 code review (m17823, finding 8) found no ruling.

BioMCP 0.9 shipped a `benchmarks/bioasq/` module that ingests public BioASQ question bundles as an offline benchmark input, with a reference page linked from its README. It is not a command, a source or a reply, so no user loses an answer when 1.0 leaves it out.

1.0 does not carry it. The release measures agent use through the blind 0.9.3 comparison (0402), MCP-Bench and InfoMosaic-Bench (0405), cBioPortal's numeric questions (0462) and the clinician question check (0490). A standing agent evaluation on each release is already listed under Later in `planning/milestones.md`; BioASQ can return there as one input if that work starts.

The 1.0 docs do not mention BioASQ. The 0.9 module and page stay readable in the published repository's history.

## Other review dispositions

Jarvis's review findings became tickets: 0519 and 0520 before the release candidate, 0521 to 0526 in milestone 6 after publication. The unused `biodata` dependency in `crates/biomcp-fetch` is removed in 0459. The BioData crate findings went to BioData's PM (m17831). No finding was declined.
