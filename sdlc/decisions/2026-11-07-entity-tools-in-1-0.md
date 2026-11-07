# Entity tools replace the typed tools in 1.0

Ian, 2026-11-07, on the MCP tools: "biomcp_reference ... seemed like it was the attic of things you never would get to. 59 tools just seemed like way too many. 20 to 25 tools feels like the right number of tools, as long as it allows you to get at all the entities ... It can't be hidden, and the number of tokens isn't really as much of a problem because you can search them ... you can shorten the tool names and you can shorten the descriptions."

Then: "I don't even need to test the 17 tool designs. Isn't it obviously better? ... I need that BioMCP 1.0 done with all the bug fixes and the new tool design before I can change hosting ... I'm not doing the deploy until BioMCP 1.0 is done with the new cleaned-up tool design."

## What it means

- The typed MCP mode in 1.0 offers one tool per thing looked up by name or ID, plus `biomcp`, help, `search` and `fetch`: about 17 tools. It replaces today's 19 typed tools.
- Each entity tool's first argument picks its action: search, get, a facet or a pivot. Each action has typed options. Links between things (GWAS associations, pharmacogenomics) are views on the tools for the things they link. Helpers (author, organization, biomarker) are options inside the tool that uses them.
- Every one of the 58 registry commands is reachable through a typed schema. Nothing sits behind a free-form object, and `biomcp_reference` goes.
- Names and descriptions are short. Token size is measured and reported, and it does not drive the shape.
- Gates: the existing replay and tool tests, plus one smoke call per tool and action.
- Hosting deploys only the 1.0 commit that carries this design.

## Reason

In the entity tools study (`records/2026-11-07-entity-tools-study.md`), about 44 commands sat behind `biomcp_reference` with no schema, and the logs showed agents guessing arguments for them. Ian judged the entity design better without a further study.

## Replaces

- The recommendation in `records/2026-11-07-entity-tools-study.md` to add the entity design in 1.1 as a new mode.
- The rule that hosted tool names stay the same 19. Hosting takes the new names with the 1.0 commit.
- 0761's deferral of the entity design to 1.1, for MCP only. On the CLI, entity-first stays an alias and today's order stays the documented form.

It keeps `decisions/2026-10-23-mcp-is-one-tool-over-the-cli.md`: the `biomcp` tool still accepts the CLI argument array, and `search` and `fetch` stay for ChatGPT.
