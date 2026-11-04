# The public benchmark gaps join 1.0

Date: 2026-10-28

Decided by Ian ("let's go ahead and get those tickets prepped and in there").

The 0405 replay of MCP-Bench's BioMCP tasks found capabilities that agents trained on biomcp-python 0.4.6 use and 1.0 lacks. They ship in 1.0, as an explicit expansion of the rule that new capabilities wait:

- NCI organization search, and NCI trials at a named organization (0408, 0409).
- NCI biomarker search, and NCI trials that list a biomarker by exact NCIt code (0410, 0411).
- NCI trial phase and status filters (0415), which the benchmark's biomarker task needs.
- FDA device recalls (0412) and drug recalls (0416). The 0162 ruling that dropped FDA recalls is replaced. 0.4.6 and 0.9 searched drug recalls and the benchmark uses neither; Ian chose both on 2026-10-28 ("I'm good with both").
- MAUDE device reports filtered by manufacturer, product code and native product problem (0413), which the benchmark's test-kit step uses.
- Several trial conditions, interventions or free-text terms joined with OR in one search (0414).

The PM checked each source on 2026-10-28 with bounded calls, and an adversarial design review corrected the tickets the same day. None needs a dataset download.

Shared rules for these tickets: keys are read only when the source is called and never printed or recorded; live captures are bounded and within each source's published limits; no licensed content in fixtures, and NCI captures prefer records with generic public contacts; new schemas use hyphenated IDs and enter the 1.0 baseline; help, examples, the generated command reference, the upgrade table and MCP tool text name the new surface; no text heuristics; a new entity that pushes the generated MCP description past its measured cap comes to the PM for a caps ruling.
