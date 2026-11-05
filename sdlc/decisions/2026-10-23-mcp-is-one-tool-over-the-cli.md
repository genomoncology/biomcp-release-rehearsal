# MCP is one tool over the CLI

Decided 2026-10-23 by Ian.

- The MCP server offers one main tool. It invokes the same searches and gets through the CLI's subcommand structure, as a service over what the CLI does.
- Applied by the PM: the tool takes the command path and arguments as a list, never a shell string. The CLI's grammar parses them and the same command functions run. The tool returns the same envelope the CLI prints.
- The tool's input and output schemas come from the same types as the CLI grammar, so the two cannot drift.
- A case may send its request through the CLI or the MCP tool, so the suite proves both agree.

Amended 2026-10-25 by Ian: the server also offers `search` and `fetch` for ChatGPT, whose deep research and company knowledge features require those two tool names. Both are thin wrappers that run the same CLI commands as the main tool, and they stay consistent with it.

Amended 2026-11-05 by Ian: typed tools ship in 1.0 as an opt-in mode. Ian: "we're going to try to get a 1.0 release out to a temporary test server, and I want that test server to be able to test the multi-tool BioMCP tool." The single tool stays the default; `mcp_tool_mode = "typed"` or `serve --tool-mode typed` offers one generated tool per command, running the same CLI commands, per `planning/typed-mcp-tools.md`. Whether typed becomes the default waits for the Biomed Agent Bench.
