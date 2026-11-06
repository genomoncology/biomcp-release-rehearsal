# Typed eager catalog: measurement and approved design

The PM's amendment to [Typed tools fit eager clients](../tickets/0683-typed-tools-fit-eager-clients.md), ruling m27799, supersedes the initial fifteen-tool / 32 KiB proposal. It selects a clinical surface from Claude's 2026-11-05 design note and the two evaluations, which used adverse events and devices. No production usage frequency ranking was available; this is an approved policy rather than a measured popularity claim.

## Baseline

Job `j31538` passed on the build machine at measurement-only revision `359fbba1873ec43944c0d854c1ef154ee3f2e3f3`, whose product source equals landed main `6b1618cc2ca4d8cf8f608e779f76ea4dfcadfa40`. `scripts/measure_typed_catalog.py` builds once and requests actual isolated offline `tools/list` replies. It measures compact UTF-8 serialization of the entire `result.tools` array, including annotations and input/output schemas, excluding the RPC envelope and instructions.

| Mode | Tools | Definition bytes |
| --- | ---: | ---: |
| Single, stdio | 3 | 10,401 |
| Typed, stdio | 61 | 72,769 |
| Hosted typed, stdio | 59 | 71,450 |

The hosted measurement is the stdio hosted policy, not an AWS deployment or a caller-specific HTTP catalog. No sources were contacted. The historical “about 58” figure understates this revision. Typed mode previously omitted the `biomcp` escape hatch.

This bounded measurement took about 249 seconds including queue/setup; Cargo reported 144 seconds for its debug build. The shared host reported load 15.57. Reported peak disk and memory were about 1.58 GiB and 0.80 GiB. These are job costs, not product latency measurements. An earlier request was cancelled while queued to correct its mode flag and never built.

## Approved defaults and reachability

The named configurable defaults are `DEFAULT_TYPED_TOOL_LIMIT = 20`, `DEFAULT_TYPED_DEFINITION_BYTES = 48 * 1024`, and `DEFAULT_TYPED_OUTPUT_DEFINITION_BYTES = 64 * 1024` when output schemas are explicitly enabled. The configuration keys are `mcp_typed_tool_limit` and `mcp_typed_definition_bytes`. One explicit byte override wins over either default, including an intentionally low override. Their single owner is `crates/biomcp/src/config/mcp.rs`. The separate 64 KiB opt-in default follows PM ruling m27869 after the measured expansions below; it does not change ordinary typed mode's 48 KiB default.

Keep direct search/get for drug, trial, article, variant, disease and diagnostic; `get_gene`; `search_adverse_event`; `biomcp_help`; research `search` and `fetch`; and `biomcp`. One `biomcp_reference` wrapper serves every other eligible registered command, including the specialist examples in the ruling, author, organization, biomarker, phenotype, specialized verbs and adverse-event lookup. This yields nineteen advertised tools. Execution policy still omits commands always refused by the effective server mode.

The [research record](2026-11-04-mcp-skills-and-design-research.md) cites Anthropic's guidance that tool search becomes useful around ten tools or 10,000 definition tokens, and recommends combining endpoint wrappers. It does not prove an optimal byte threshold. The PM chose twenty tools / 48 KiB to preserve the named clinical surface and complete descriptions. Bytes are measured separately from tokens; neither this cap nor the lower tool count proves model accuracy or a token count.

## One membership and caller-policy owner

`typed/grouping.rs` retains one immutable catalog plan containing both the advertised tools and the original eligible, caller-filtered command definitions. The latter are private server state, not hidden bytes in the advertised JSON. Build this plan once per typed reply. Follow-up translation checks canonical arguments against their original caller-specific schema, then maps the call to an advertised direct tool or `biomcp_reference`. It does not rebuild a catalog per suggested step or keep another ranking.

The reference wrapper accepts a registered command name and an object of typed arguments. Dispatch unwraps it into the existing typed validator and CLI converter. Unadvertised historical names remain accepted for compatibility, while generated follow-ups use advertised routes. Single mode's definitions and CLI replies remain unchanged; typed mode remains opt-in.

Measure the complete catalog with the complete description lane incorporated. If it exceeds the default cap, report the measured composition to the PM; never evict a clinical tool or truncate prose. Explicitly undersized custom budgets return an actionable error naming the required tools/bytes and the configured ceilings. Optional output schemas and larger enums count toward the configured cap too.

## Proof boundaries

Existing installed discovery cases prove complete registry reachability through direct or reference routes and exact count/serialized-byte boundaries. Existing pathway parity witnesses use the grouped wrapper; the historical recorded parity helper likewise routes grouped commands through the advertised wrapper. Existing hosted cases preserve execution refusals and caller source exclusions. Shared follow-up tests validate actual advertised schemas. The routine gate covers formatting, lint, docs and the complete local suite. No deployment, release candidate, publication or new data source is part of this change.

## Measured configured profiles

Job `j31622` measured the complete catalog at `c2bde9975c11a05aac332bd18068cdd5788a658f`, including complete description changes and caller-aware typed follow-ups. The enum-case hint changes were not composed. Every typed profile has nineteen tools. Expansion profiles explicitly used `mcp_typed_definition_bytes: 200000` for measurement; that is a diagnostic override, not a product default.

| Output schemas | Enum limit | Stdio definition bytes | Hosted stdio definition bytes |
| --- | ---: | ---: | ---: |
| Off | 1 | 33,798 | 33,765 |
| Off | 30 (normal default) | 36,667 | 36,634 |
| Off | 1000 | 39,486 | 39,453 |
| On | 1 | 55,226 | 55,193 |
| On | 30 | 58,095 | 58,062 |
| On | 1000 | 60,914 | 60,881 |

The ordinary default profiles succeeded with the actual default 49,152-byte ceiling. Excluding ctgov, who-pq and gwas measured 36,580 bytes. Single mode measured three tools / 10,631 bytes; its 230-byte growth from the original baseline comes from the shared research descriptions, not the grouping.

All measured output-schema expansion profiles exceed 49,152 bytes. This was reported for a PM ruling without changing the production default, grouping a clinical tool or truncating descriptions. The measurement preceded the focused cases: 24 passed, while the description-contract case explicitly enabled output schemas without a larger budget and therefore received the expected catalog-budget error. Its generic “tools” failure obscured that configuration cause.

Raw profile configurations, compact definitions, RPC replies, binary binding, the full job log and transcripts were copied to the separate ticket evidence store on Yellow under this exact revision / `j31622-red`. The receiver verified every file against the sender's SHA-256 manifest. The original baseline and the earlier obsolete-80-character-assertion failure are stored separately. No baseline was remeasured.


PM ruling m27869 approved the measured distinction: ordinary typed discovery keeps the 48 KiB default; explicitly enabling output schemas selects a named 64 KiB default. The single explicit byte override takes precedence in both modes. The measured roughly 55–61 KB expansions fit that opt-in default; they are not ordinary-profile cap breaches. The final profile matrix uses these production defaults without the diagnostic 200000-byte override. No clinical tool or description changes to achieve either cap.

Job `j31636` subsequently passed all 25 focused MCP cases and the sixteen-profile matrix at `b0ceec8744eb5229511ad4d44f4a4e6674b67164`, using the actual 48/64 KiB production defaults with no diagnostic byte overrides. The byte counts in the table remained unchanged. Explicit undersized count/byte settings fail, and a low explicit byte setting still wins when output schemas are enabled. The complete green evidence is separately retained under that revision / `j31636-green`; all 264 files, including every raw profile definition/configuration, binary binding, RPC replies, transcripts and terminal log, were receiver-verified against the sender manifest. The enum-case hint changes remain uncomposed. The following source-only string layout correction preserves the catalog text while satisfying the repository line-width boundary; it does not claim another size measurement.
