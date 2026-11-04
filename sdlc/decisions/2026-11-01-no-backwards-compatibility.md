# No backwards compatibility in the CLI and MCP

Decided by Ian on 2026-10-31, relayed by Jarvis on 2026-11-01 (m18009).

Ian's words: "I have no interest in backwards compatibility." He said the same of BioData. Later that day he added: "The CLI and MCP for now in BioMCP has no backwards compatibility promises."

As Ian confirmed, BioMCP carries no compatibility code: no readers or migrations for old formats, no aliases or no-op flags for old names, no fallbacks for old config, and no code lists kept open for old clients. The release notes list every removed or changed command, option, config key and output field, so users know what to change. 0.9 users get that list and nothing more.

This ruling supersedes the team's own freeze reasoning. The deferral line in `2026-10-27-1-0-ships-every-known-fix.md` that waits for the next schema major cites a freeze Ian never ruled, and plan text that says the release gate freezes patterns or type sets describes the team's design, not a promise to users. Schema IDs keep their versions, so a reader can refuse a version it cannot read. The MCP protocol revisions BioMCP answers serve current clients of an outside protocol, and this ruling does not decide them.
