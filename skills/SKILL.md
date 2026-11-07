---
name: biomcp
description: Retrieve biomedical source records through the BioMCP command line and MCP server.
---

<!-- {{front-map}} -->

If no command map appears above, run `biomcp skill install DIR` to write the full guide for your BioMCP version.

Use `biomcp help COMMAND ENTITY` for syntax and `biomcp skill list` for examples. Follow returned next commands for pages and sections. Preserve source identifiers, dates, terms and uncertainty. BioMCP retrieves source records and does not choose treatments or run bioinformatics tools.

## MCP typed tools

In typed mode each entity tool takes `action` and one object named after it. The tools are `gene`, `variant`, `disease`, `drug`, `trial`, `article`, `protein`, `pathway`, `phenotype`, `adverse_event`, `diagnostic`, `cell_line` and `study`, plus `biomcp` (the CLI arguments), `help`, `search` and `fetch`. Use `search` to find records and `get` to read one. Other actions are facets, pivots and links to related records, such as `gwas` or `pgx`. A tool's description lists its actions. To read BRAF, call `gene` with `{"action":"get","get":{"input":["BRAF"]}}`. Call `help` with a command path for options.

## Exit codes and errors

<!-- exit-reference:start -->
Process exit codes:
0: complete or partial reply
1: no matching record
2: invalid command or arguments
3: expired result pool
4: source request failed
5: local configuration, storage or output failed
Error codes: usage.schema, usage.arguments, usage.mcp_unavailable, usage.unknown_command, usage.retired, local.failure, pool.expired, usage.pool_mismatch, usage.too_many_pmids, usage.too_broad, usage.unsupported_filter. Source problems and reasons appear in their reply fields.
<!-- exit-reference:end -->
