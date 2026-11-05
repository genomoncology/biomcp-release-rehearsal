# MCP, skills, and how to know BioMCP's MCP design is as good as it can be

Research for Ian, 2026-11-04. Every source below was accessed on 2026-11-04 unless marked otherwise. Each claim is tagged **[Verified]** (read in a primary source: the spec, a SEP, the official repo, or the client's own docs), **[Secondary]** (blogs, papers' abstracts read through a summary, search summaries, forum posts), or **[Not found]** (looked and could not confirm).

## The short answer on skills

- **Skills are now official MCP, but as an optional extension, not part of the core spec.** The extension is called "Skills over MCP" (identifier `io.modelcontextprotocol/skills`). It was proposed as SEP-2640 (a SEP is a "spec enhancement proposal"), created 2026-04-23, merged 2026-09-13, and is marked Final. **[Verified]** [SEP-2640](https://modelcontextprotocol.io/seps/2640-skills-extension), [PR #2640](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2640), [ext-skills README](https://github.com/modelcontextprotocol/ext-skills).
- **Its maturity is "Beta".** Since SEP-3392 (Extension Lifecycle, 2026-10-29), extensions move Experimental → Beta → Stable → Core. Beta means: "Real usage is welcome. Breaking changes are documented." The skills spec now lives at `specification/beta/skills.mdx`. **[Verified]** [SEP-3392](https://modelcontextprotocol.io/seps/3392-extension-lifecycle), [ext-skills commit 2026-10-29](https://github.com/modelcontextprotocol/ext-skills/commits/main).
- **A skill over MCP is exactly an Agent Skills folder (a `SKILL.md` plus optional files), served as MCP resources.** The extension does not invent a new format. In its words: "The skill format itself ... is delegated entirely to the Agent Skills specification; this SEP defines only the transport binding." **[Verified]** [SEP-2640 abstract](https://modelcontextprotocol.io/seps/2640-skills-extension).
- **A competing design, skills as a brand-new primitive like tools or prompts (SEP-2076), was closed.** The working group chose resources because "Skills are files; Resources exist to expose files", and a new primitive would have flattened skills "to name-addressed blobs", losing the folder of supporting files. **[Verified]** [rationale.md](https://github.com/modelcontextprotocol/ext-skills/blob/main/docs/rationale.md), [SEP-2076 PR](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2076).
- **Almost no mainstream client loads MCP-served skills yet.** On the official support matrix (last edited 2026-11-03), Claude (web), Claude Desktop, Claude Code, Cursor and VS Code have no Skills mark. ChatGPT is "Partial". Full marks go to small clients: MCPJam, mcpc, Glean; fast-agent and MCP Inspector are "Partial". VS Code is "planned" and Goose and opencode are "in progress". **[Verified]** [client matrix source](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/extensions/client-matrix.mdx), [implementations list](https://github.com/modelcontextprotocol/ext-skills/blob/main/docs/implementations.md).
- **The Rust SDK BioMCP uses (rmcp) does not ship it yet:** "Rust SDK (official): in progress (#1392)". TypeScript ships it; Python and C# have merged it unreleased. **[Verified]** [implementations list](https://github.com/modelcontextprotocol/ext-skills/blob/main/docs/implementations.md).
- **BioMCP already does an informal version of this.** `biomcp serve` lists `biomcp://help` and one `biomcp://skill/<slug>` resource per use-case playbook, and its server instructions point to `biomcp skill list`. It does not declare the extension or answer `skills/list`, and its URIs don't follow the `skill://<name>/SKILL.md` shape. **[Verified]** `repos/biomcp/src/mcp/shell.rs` (`build_resource_list`), `src/mcp/catalog.rs` (`instructions`).

## How the skills extension works

All quotes are from [SEP-2640](https://modelcontextprotocol.io/seps/2640-skills-extension) **[Verified]** unless noted.

**What the server publishes.** Each file in a skill folder is a resource, normally at `skill://<skill-path>/<file-path>`, so the main file is always `skill://<name>/SKILL.md`. The last path segment must equal the `name` in the SKILL.md frontmatter. The `SKILL.md` must start with YAML frontmatter holding `name` (up to 64 lowercase characters) and `description` (up to 1,024 characters, saying what the skill does and when to use it). [Agent Skills spec](https://agentskills.io/specification).

**How the server says it supports skills.** In the 2026-07-28 protocol, it declares `resources` plus `"extensions": {"io.modelcontextprotocol/skills": {}}` in its `server/discover` reply. A server that declares it "MUST implement `skills/list` and `skills/get`".

**How a client finds and loads skills.** There are three routes:
1. `skills/list` returns, for every skill, its URI, its whole frontmatter, and a full manifest of its files with a SHA-256 digest and size each. The listing is designed to be enough on its own: "there is no second round-trip per skill".
2. `skills/get` returns one skill's entry by URI, including skills a partial listing left out.
3. A pointer in the server's instructions: "A server MAY direct the agent to specific skill URIs from its `instructions` field. This requires no discovery machinery on the host."

Then the client reads files with the ordinary `resources/read`. An optional `resources/directory/read` lists a folder.

**How it relates to progressive disclosure** (showing the model a little first, and more only when needed). The host puts only each skill's name and description in the model's context. The full SKILL.md loads when the skill is activated, and supporting files load only when read. The Agent Skills spec suggests about 100 tokens per skill for the first level, a SKILL.md under 5,000 tokens and 500 lines, and references one level deep. **[Verified]** [Agent Skills spec](https://agentskills.io/specification). The host "MUST NOT fetch `SKILL.md` or any supporting file" when it builds its list.

**How it relates to the other MCP parts.**
- *Tools* do the work; skills are workflow text about using them.
- *Resources* are the transport for skills.
- *Server instructions* are the always-on summary and may point to skill URIs. The SEP's case for skills is that instructions "are practically bounded in size", so long workflows do not fit there.
- *Prompts* (templates a user picks, often as slash commands) are separate. The SEP does not use them.
- **Reading is not loading.** "Reading a `SKILL.md` via `resources/read` does not by itself activate the skill." Only the host's own skill path activates it, after checking digests and any user approval.

**Security rules that shape how hosts treat a server's skills.** Hosts must treat skill text as "untrusted input", label it with the server it came from, and never let a skill run code on the user's machine without "explicit per-skill user approval". They must ignore the Agent Skills `allowed-tools` field from a server unless the user approves it. A skill from server A must not cause reads from server B. Any approval is tied to the file digests, so a changed skill must be approved again. Limits are 512 files and 16 MiB per skill. For BioMCP, this means a skill should be plain guidance about BioMCP's own tools, with no bundled scripts, because scripts would trigger approval prompts in every careful host.

**What may be on Ian's mind.** The extension is new as of September, and three related things are easy to blur together. Anthropic's Agent Skills format is agentskills.io. Claude Code ships skills inside plugins next to an MCP server; BioMCP already has a `.claude-plugin`. OpenAI's ChatGPT and Codex plugins also bundle skills with an MCP server ([OpenAI, Build skills](https://developers.openai.com/plugins/build/skills)) **[Verified]**. The matrix marks ChatGPT's support as "partial, static snapshot": it imports skills from the server once, when the plugin is submitted, and does not fetch them live. The current English docs page no longer shows that section, so this rests on the matrix plus a search summary of translated pages and a forum post ([forum](https://community.openai.com/t/support-skills-over-mcp-on-custom-plugin/1394917)) **[Secondary]**.

## Evidence that skills actually get used

This is the weak spot, and it matters more than the wire format.

- **The working group's own findings:** "Models appear to frequently ignore available skills"; following them fades as the context grows; and weaker models do worse with skills loaded on demand. In the VS Code prototype, discovery worked, but "no run yet shows the model choosing to load an MCP-served skill". **[Verified that the group records this; the reports themselves are anecdotes]** [experimental-findings.md](https://github.com/modelcontextprotocol/ext-skills/blob/main/docs/archive/experimental-findings.md).
- **mcpGraph:** Claude ignored a skill that sat beside the server until one server instruction told it to read the skill first, and then it loaded the skill "reliably". **[Secondary, anecdote recorded in the same file]**.
- **Vercel (2026-01-27):** with an on-demand docs skill, "In 56% of eval cases, the skill was never invoked". Pass rates were 53% with no docs, 53% with the skill, 79% with the skill plus an instruction to use it, and 100% with an 8 KB docs index always in context. **[Verified]** [Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals).

Both point the same way as our 513 finding: a one-page map placed in the prompt cut tokens by about 20%. Whatever the model sees without having to decide to look tends to beat whatever it has to choose to load.

## Other 2026 MCP changes that bear on design

The current spec is **2026-07-28**, released 2026-07-28. It replaced 2025-11-25. **[Verified]** [versioning](https://modelcontextprotocol.io/specification/versioning), [changelog](https://modelcontextprotocol.io/specification/2026-07-28/changelog), [releases](https://github.com/modelcontextprotocol/modelcontextprotocol/releases).

| Feature | What it is (2026-07-28) | Helps BioMCP? |
|---|---|---|
| Stateless protocol, `server/discover` | No handshake. Every request carries version and capabilities, and servers "MUST implement" `server/discover`. **[Verified]** | Required plumbing. BioMCP already speaks 2026-07-28 next to 2025-06-18 and 2025-11-25 (`src/mcp/shell/modern.rs`). |
| Server instructions | Server-wide text for the model. Official advice: say how tools relate, the order to call them, and limits; don't repeat tool descriptions; keep it short. **[Verified]** [MCP blog, 2025-11-03](https://blog.modelcontextprotocol.io/posts/2025-11-03-using-server-instructions/) | Yes, the most widely read channel. But clients cut it differently (see below). |
| `outputSchema` / `structuredContent` | A tool can declare a JSON shape for its output and return matching JSON. It "SHOULD also return the serialized JSON in a TextContent block". 2026 allows any JSON Schema 2020-12 keyword and any JSON value. **[Verified]** [tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools) | Likely yes in code mode. pi hands `structuredContent` to scripts, so a script can filter fields without the model reading them. Little gain for plain tool calling. Worth testing. |
| Tool annotations | `readOnlyHint`, `idempotentHint`, `openWorldHint`, `destructiveHint`, which clients treat as untrusted hints. **[Verified]** | Yes, at almost no cost. Claude Code runs read-only calls in parallel and retries them after a dropped connection ([Claude Code MCP docs](https://code.claude.com/docs/en/mcp)), and ChatGPT and Codex use them for confirmations ([OpenAI](https://developers.openai.com/plugins/build/mcp-server)). **[Verified]** |
| Resource links, embedded resources | A tool result can link to a resource or include its full text. **[Verified]** | Maybe. One route to return a big record as a link, read on demand. Untested. |
| Deterministic tool order, `ttlMs` / `cacheScope` | Servers "SHOULD return tools ... in a deterministic order", which helps prompt caching. List results carry cache hints. **[Verified]** | Yes, cheap. |
| Error results | Execution errors go back with `isError: true` and "actionable feedback that language models can use to self-correct". **[Verified]** | Yes, already a BioMCP theme (`structured_error.rs`). |
| Elicitation (asking the user a question through the client) | Now goes through "multi round-trip requests": the server answers "input required" and the client retries. **[Verified]** | Little. A lookup server rarely needs to ask the user. |
| Sampling, Roots, Logging | Deprecated in 2026-07-28. **[Verified]** | Don't build on them. |
| Tasks (long-running jobs) | Moved out of the core spec into a Beta extension, `io.modelcontextprotocol/tasks`, with polling. Claude Code and claude.ai support it. **[Verified]** [changelog](https://modelcontextprotocol.io/specification/2026-07-28/changelog), [matrix](https://modelcontextprotocol.io/extensions/client-matrix) | Only for slow batch jobs. Not now. |
| MCP Apps (UI shown inside the chat) | A Stable extension, supported by most big clients. **[Verified]** | Not for an agent-facing data server. |
| Tool search / grouping | **Not in the spec.** A tool search SEP (SEP-1821) and a grouping SEP (SEP-2084) were closed. A Progressive Disclosure Working Group (renamed 2026-09-21) is exploring "grouping, tool-search, code-mode" and has no recommendation yet. **[Verified]** [charter](https://modelcontextprotocol.io/community/working-groups/progressive-disclosure) | Tool search lives in clients, so test it client by client. |

**Code execution patterns** (the model writes code that calls the tools, instead of calling tools one at a time):
- Cloudflare's "Code Mode" (2025-09-26) turns MCP tools into a TypeScript API that runs in a sandbox. **[Verified]** [Cloudflare](https://blog.cloudflare.com/code-mode/)
- Cloudflare's later API server exposes more than 2,500 endpoints through two tools, `search()` and `execute()`, in about 1,000 tokens. **[Secondary]** [InfoQ](https://infoq.com/news/2026/04/cloudflare-code-mode-mcp-server)
- Anthropic's "Code execution with MCP" (2025-11-04) cut one example from 150,000 to 2,000 tokens. **[Verified]** [Anthropic](https://www.anthropic.com/engineering/code-execution-with-mcp)
- Anthropic's "Advanced tool use" (2025-11-24): tool search raised Opus 4 from 49% to 74% on MCP evals. Calling tools from code cut tokens 37% on research tasks. Examples in tool definitions raised accuracy on complex parameters from 72% to 90%. Tool search is worth it at "10+ tools" or more than 10K tokens of definitions. **[Verified]** [Anthropic](https://www.anthropic.com/engineering/advanced-tool-use)
- pi's code mode is the case we measured.

## What each client does

| Client | Skills over MCP | Server instructions | Resources | Prompts | Structured content | Tool search / deferral |
|---|---|---|---|---|---|---|
| Claude Code | No (matrix). Skills come in through plugins instead. | Yes. Cut at 4,096 characters; tool search reads them to decide when to search. | Yes (`@server:uri`, plus list and read tools) | Yes (as slash commands) | Not mentioned in docs | On by default. Only tool names and instructions load at start. `_meta["anthropic/alwaysLoad"]` makes a tool load up front. |
| claude.ai / Desktop | No (matrix). Native skills exist, but not over MCP. | [Not found] | Tools, prompts and resources, per a support article that now returns 404 **[Secondary]** | Same | [Not found] | [Not found] |
| Codex | Partial, through ChatGPT plugin import | Yes ("Codex reads the MCP `instructions` field") | [Not found] | [Not found] | [Not found] | [Not found]. It has `enabled_tools` and `tool_timeout_sec`. |
| Cursor | No (matrix) | [Not found] | Yes | Yes | [Not found] | [Not found] |
| VS Code | Planned | Yes | Yes | Yes | [Not found] | [Not found] |
| pi 1.4.0 docs (we ran 1.5.0) | No. pi has native Agent Skills only. | Only one line in the system prompt; the full text only via `describeNamespace()` | Yes (`list_mcp_resources`, `read_mcp_resource`) | Not in docs | Yes, scripts receive `structuredContent` | Default "codemode" exposure. Also "deferred" (`tool_search`) and "direct" (declared to the model). |

Sources, all **[Verified]**: [Claude Code MCP](https://code.claude.com/docs/en/mcp), [Codex MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli), [Cursor MCP](https://cursor.com/docs/context/mcp), [VS Code MCP](https://code.visualstudio.com/api/extension-guides/ai/mcp), and pi's `docs/mcp.md` and `docs/codemode.md` from the local install. The one exception is the claude.ai row, which comes from a [search summary](https://support.claude.com/en/articles/11503834-building-custom-connectors-via-remote-mcp-servers) **[Secondary]**.

**The key design fact:** the same server instructions reach the model three different ways. Claude Code shows the first 4,096 characters every session. pi shows one line, and the rest only if the model calls `describeNamespace`. Skills over MCP reach almost no one yet. So a design that works in one client may not carry to another, and the test plan has to cover more than pi. 513's map is about 4.0 KB (`api-map.md`, 3,991 bytes) and the terse map about 2.9 KB, so either fits Claude Code's instruction limit.

## What is known about good tool design

- **Anthropic, "Writing effective tools for agents" (2025-09-11)** **[Verified]** [post](https://www.anthropic.com/engineering/writing-tools-for-agents):
  - "More tools don't always lead to better outcomes."
  - Don't just wrap API endpoints; combine steps into fewer tools.
  - Put a shared prefix on related tool names.
  - Return only high-signal fields, with a concise/detailed option (concise used about a third of the tokens).
  - Paginate and truncate with sensible defaults.
  - Make errors actionable, and write descriptions as for a new hire.
  - Evaluate on realistic tasks with a held-out set, tracking accuracy, tool calls, tokens, errors and run time.
- **Tool count against accuracy:**
  - RAG-MCP (2025-05) more than tripled tool-picking accuracy (13.62% to 43.13%) by retrieving tools instead of listing them all. **[Secondary, abstract]** [arXiv 2505.03275](https://arxiv.org/abs/2505.03275)
  - "How Many Tools Should an LLM Agent See?" (2026-05) found that shorter shortlists chosen per question beat a fixed list. **[Secondary, abstract]** [arXiv 2605.24660](https://arxiv.org/abs/2605.24660)
  - MCPVerse finds most models get worse as tool count grows. **[Secondary]** [arXiv 2508.16260](https://arxiv.org/html/2508.16260v1)
- **MCP against CLI token cost:** a widely cited ScaleKit benchmark reports the CLI 4–32× cheaper, while a Smithery study found MCP more reliable. Blocks.ai argues the real factor is loading schemas up front against loading them on demand. None of this is peer reviewed. **[Secondary]** [Firecrawl summary](https://firecrawl.dev/blog/mcp-vs-cli), [Nordic APIs](https://nordicapis.com/?p=23433), [Blocks.ai](https://blocks.ai/blog/mcp-vs-cli-context-window-cost)
- **[Not found]:** any published study that compares skills over MCP, server instructions, and a map as a resource side by side. Our test would be among the first.

## Test plan for BioMCP

**The question:** which mix of tool shape and help delivery gets every field right for the fewest tokens, across the clients people actually use?

**Arms (design options).** Each arm changes one thing from A.

- **A. One CLI-args tool + current instructions.** The winner so far; the baseline.
- **B. Typed tools.** Both the 7 typed tools now on main and 513's 14-tool variant, to see whether count or shape matters.
- **C. A + map in the instructions.** Map first, under 4,096 characters, with the first sentence written to survive pi's one-line summary.
- **D. A + map as a resource.** The map at `biomcp://map`, with a one-line pointer in the instructions.
- **E. A + skills, done properly.** Declare the extension, answer `skills/list` and `skills/get`, and serve `skill://biomcp/SKILL.md` with the use-case playbooks as `references/` files. Point to it from the instructions. Needs a hand-written handler until rmcp ships support.
- **F. A + `outputSchema` / `structuredContent`.** Code-mode clients only.
- **G. A + `readOnlyHint` and deterministic ordering.** Expected to be neutral on tokens; this checks for no harm.

Arms E and D are only worth keeping if the agent actually opens the skill or map. So log the **activation rate** (did it read the map or skill?) next to tokens.

**Clients:**
- **pi 1.5.0 with Terra.** Run in both code-mode exposure and direct exposure, since that is where cost differed.
- **Claude Code.** Tool search on by default; the most common real client; it reads instructions in full.
- **Codex CLI.** It reads instructions; OpenAI is the other big host.
- **For arm E only, one client that implements skills over MCP:** fast-agent (Partial) or mcpc (full), to measure real `skills/list` loading. In pi and Claude Code, E falls back to the instruction pointer plus resource read, which is the route the spec guarantees everywhere.
- **Leave out for now:** VS Code, Cursor and claude.ai. They are hard to script, and nothing in them is unique to this question.

**Method (513's):**
- The same fixed questions: the climb, check and held-out sets.
- A cold agent each run: a new session with no memory and no map unless the arm supplies one.
- At least two repeats per cell, compared by median.
- A change counts only if it beats the baseline by more than about 30%, or by twice the measured spread between repeats, whichever is larger.
- Score tokens (input plus output, with cache reads reported apart) and fields lost against the curated answers, kept away from the agent under test.
- Keep infrastructure errors apart from model mistakes.
- Rival: the cold native harness (`terra-scripts`).

**Order:**
1. C against D against A in pi and Claude Code. Cheap, and it settles where the map goes.
2. E, once C/D show whether help gets read at all.
3. F in pi.
4. Repeat the best two arms on the next model.

**When we can say "as good as we can":**
- The chosen design beats the cold native harness on fields and tokens in at least two clients, on the held-out set.
- Every feature that remains earned its place in an arm, beyond noise.
- The server passes MCP Inspector, including its skill check.
- Every feature dropped is written down with the result that dropped it.

## Gaps

- I read pi's 1.4.0 docs locally. Behaviour in 1.5.0 may differ.
- I could not confirm whether claude.ai and Desktop pass server instructions or structured content to the model.
- Codex docs are silent on resources and structured content.
- Neither Anthropic nor OpenAI documents a date for loading MCP-served skills live.
