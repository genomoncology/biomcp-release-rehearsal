# BioMCP

BioMCP retrieves biomedical evidence for cancer care and human health research. It connects genes, variants, drugs, diseases, trials, articles and other entities to their source records through a command line and the Model Context Protocol (MCP). Results retain source identities, terms, problems and next commands. BioMCP does not make treatment decisions or run bioinformatics tools.

## Install and upgrade

Install the published package with `uv tool install biomcp-cli`. Upgrade an existing installation with `uv tool upgrade biomcp-cli`. Use the package manager or verified installer that owns your installation for updates and removal. BioMCP has no self-update command.

Read the [1.0 release notes](release/notes.md) before upgrading. The [upgrade guide](sdlc/planning/upgrade.md) maps 0.9 commands to their replacements and explains retired capabilities. Confirm the installed version before using this command reference:

```sh
biomcp version
```

## Configure sources and keys

Every source and component field is enabled by default, including noncommercial and separately licensed data. Publisher terms remain authoritative. Public API access grants no additional reuse rights. Commercial users must set this configuration:

```json
{"disable_sources": ["noncommercial", "licensed"]}
```

BioMCP reads `$XDG_CONFIG_HOME/biomcp/config.json` or `~/.config/biomcp/config.json` on Unix, and `%APPDATA%/biomcp/config.json` on Windows. Use `--config PATH` for another file. Source names and the `noncommercial` and `licensed` groups also work with `--disable-sources`. Configuration and command exclusions combine. Exclusions also prevent disabled data from returning through cache, replay or stored collections.

See [server configuration](docs/configuration.md) for cache and store folders, Docker storage and server settings.

Use `biomcp key set oncokb` to enter your own key at a prompt that hides input. Keys stay in the configuration folder and out of shell history. `biomcp key list` reports credential names and origins without values. Source-specific environment variables remain available; `biomcp help key oncokb` explains the supported names. A source that requires your account or license sends no request without your key. An optional key raises its source's limit; the source remains available at its public limit without it.

For company networks, `ca_bundle` adds PEM certificates to platform trust and keeps verification enabled. `proxy` sets an explicit HTTP or HTTPS proxy. Otherwise BioMCP prefers `https_proxy`, `http_proxy` and `no_proxy` over `HTTPS_PROXY`, `HTTP_PROXY` and `NO_PROXY`. Use `biomcp help network ca_bundle` or `biomcp help network proxy` for details.

## Ask a question

Which source record identifies KRAS?

```sh
biomcp get gene KRAS
```

Which PubMed articles match AKT1 E17K?

```sh
biomcp search article 'AKT1 E17K'
```

Which registered trials match melanoma?

```sh
biomcp search trial --condition melanoma
```

Trial and article searches expand a whole drug name to the other names stated by an existing drug source when it identifies one drug. Source notices list the names searched and any left out by the limit. Ambiguous names stay as typed, and a reply says when lookups failed and names were not expanded. Use `--exact-term` to skip this expansion; the selected provider still controls its own matching and ranking.

These commands come from recorded cases. Live replies can change. Inspect source outcomes before interpreting a result. Follow returned next commands to page records, inspect source sections or resolve an ambiguous identity. A missing result does not establish absence.

JSON is the default output. Use `--format md` for readable Markdown with tables and copyable next commands, or set `"format": "md"` in configuration. MCP always returns JSON. The [command reference](docs/reference/commands.md) lists every command, option and section. The [worked commands](docs/recipes.md) provide more examples. The [source and entity cautions](docs/cautions.md) explain interpretation limits.

Some sources use local bundles. Missing stored bundles offer an explicit sync command. HGNC and ClinGen can synchronize missing bundles within the configured read budget. Live servers return stale HGNC and ClinGen rows immediately, marked stale, while refreshing in the background. One-shot commands and `--fresh` reads wait for refresh. Other stored sources require explicit synchronization. Offline mode never synchronizes. Stale stored rows remain readable with a stale marker.

## Connect an MCP client

Run `biomcp mcp-config` to print ready-to-paste stdio configuration using the installed executable's absolute path. Add that configuration to your MCP client. The client starts `biomcp serve` and calls the `biomcp` tool with the same command grammar as the CLI. Read-only research clients can also use `search` and `fetch`; search returns opaque result IDs for fetch.

For native HTTP clients, `biomcp serve --http` listens at `http://127.0.0.1:8080/mcp`. Supply an address after `--http` to select another listener. A non-loopback listener requires `--allow-host HOST` or `http.allowed_hosts` in configuration. The allowlist does not authenticate callers. Restrict listener access to trusted clients. Browser origins are refused. HTTP refuses synchronization, store cleaning, fresh requests and file saves. Stdio allows new-file saves and refuses replacement. `GET /health` checks the local server and store without upstream requests.

See [hosted server configuration](docs/configuration.md#hosted-servers) for deployment, source use, caller credentials and request limits.

## Develop

The documentation check does not run the exact `uv tool install biomcp-cli` or `uv tool upgrade biomcp-cli` shell instructions.

Run `make docs-generate` after command changes and `make docs-check` to check generated documentation and execute documented examples against recordings. Run `make landing-gate` for the short landing check: formatting, lint, rights, dependency and secret checks, then the CLI and MCP cases. See [ROADMAP.md](ROADMAP.md) for scope and source availability.

`make schema-release-check` compares generated schemas with the portable snapshot named in `release/schema-baseline.json`. It enforces schema major-version compatibility and rejects missing, malformed or incomplete baselines. Release artifact checks install and exercise the produced package in an isolated environment.
