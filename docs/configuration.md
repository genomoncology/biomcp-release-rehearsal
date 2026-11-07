# Server configuration

BioMCP reads the configuration file described in the README. Unknown keys cause a configuration error. Command flags override their configuration keys.

## Cache and store folders

On Unix, BioMCP keeps its cache and stored collections in `$XDG_CACHE_HOME/biomcp`, or `~/.cache/biomcp` when `XDG_CACHE_HOME` is unset. On Windows, it uses `%LOCALAPPDATA%/biomcp`. Choose a local disk for these folders. Sharing one cache between several processes through a Docker Desktop host bind mount or a network filesystem is unsupported. BioMCP uses SQLite's write-ahead log (WAL), which needs shared memory between processes on the same host; see the [storage caution](cautions.md#cache-and-store-storage).

Set `BIOMCP_SQLITE_DIAGNOSTICS` to an existing file to opt into bounded SQLite operation and numeric error-code diagnostics (up to 4096 bytes); it does not include queries or patient data.

### Docker

Mount a Docker named volume at the container's `XDG_CACHE_HOME` path to keep the cache and stored collections on container-native storage. The published image uses `/tmp/cache`. Use a volume writable by the image's user, UID 65532. Sharing one cache between several processes through a Docker Desktop host bind mount or a network filesystem is unsupported; use a local named volume instead.

## MCP tools

`biomcp serve --tool-mode typed` advertises direct search/get tools for drug, trial, article, variant, disease and diagnostic, plus gene lookup and adverse-event search. Other commands use `biomcp_reference` with a registered `command` name and its typed `arguments` object. For example, `{"command":"get_protein","arguments":{"id":"P04637"}}` reads a protein. `biomcp_help` supplies command contracts and examples, and the `biomcp` tool retains the complete CLI grammar. Research `search` and `fetch` remain available. Typed calls return structured replies and a one-line summary. Set `"mcp_tool_mode": "typed"` in configuration for the same behavior. The flag overrides configuration. `single` remains the default and keeps the CLI-argument tool and research compatibility tools. Hosted restrictions apply in either mode. Typed mode follows `mcp_reply_form`: text begins with a one-line summary and includes the serialized JSON when the chosen form carries JSON text. The result cap counts the JSON value once in either tool mode. Typed `next` entries carry `kind`, `name` and `arguments`. Sections use the command's `section` field, including inside a reference call's nested arguments. Replies omit inherited CLI globals and server paths from these calls, and omit suggestions that the effective catalog cannot execute.

Set `"mcp_typed_output_schema": true` to include reply envelope schemas on the direct entity tools and `biomcp_reference`. The shared `biomcp`, research `search` and `fetch` tools retain their output schemas in either setting. `mcp_typed_enum_limit` sets the largest choice list advertised inline (default 30); larger lists use strings, with accepted values checked by the CLI parser. Both settings apply only to typed discovery; the reference command-name enum always lists its routes. For example:

```json
{
  "mcp_tool_mode": "typed",
  "mcp_typed_output_schema": true,
  "mcp_typed_enum_limit": 30
}
```


`mcp_typed_tool_limit` and `mcp_typed_definition_bytes` set positive ceilings on the complete typed catalog. Defaults are 20 tools and 48 KiB of compact UTF-8 tool definitions, or 64 KiB when output schemas are explicitly enabled; their owner is [the typed budget configuration](../crates/biomcp/src/config/mcp.rs). Counts include help, research, reference and escape-hatch tools. Bytes include descriptions, annotations and any output schemas, excluding the JSON-RPC envelope and server instructions. A catalog exceeding either ceiling returns an error naming the required size and configured limits. Clinical tools and complete descriptions are retained. An explicit `mcp_typed_definition_bytes` value overrides either default, including when output schemas are enabled. Larger catalogs may require a larger configured byte ceiling. Single mode is unaffected.

## HTTP capacity

`http_max_calls` limits tool calls in flight. `http_max_connections` limits open non-health HTTP connections. Each key accepts a positive integer. The server refuses `http_max_connections` when it is below `http_max_calls`. Pending framing readers and active requests share the connection cap. Framing readers share the existing worker bound of twice the connection setting plus one. `/health` uses neither the connection cap nor the tool call cap, but idle or incomplete sockets can exhaust framing workers and make it return 503. A front end that forwards complete requests avoids this raw-socket limit.

Use the same names as flags on `serve --http`: `--http_max_calls` and `--http_max_connections`. The defaults live in [`DEFAULT_HTTP_CAPACITY`](../crates/biomcp/src/config.rs). Raise connections above calls to let discovery requests answer while tool calls occupy their slots.

For example, this configuration gives discovery requests room alongside tool calls:

```json
{
  "http_max_calls": 32,
  "http_max_connections": 48
}
```

The equivalent flags are:

```sh
biomcp serve --hosted --http --http_max_calls 32 --http_max_connections 48
```

The `--rate-share` flag scales only hosted buckets with an effective source key. Keyless hosted buckets and ordinary process buckets keep the source's full rate.

## Source pacing

`source_pacing` is one map keyed by provider bucket name. The [generated source reference](reference/source-pacing.md) lists every name, default, ceiling and policy citation. Shared sources such as the BioThings APIs use one bucket.

Each map entry can set `rate`, `keyed_rate` and `in_flight`. `rate` means requests without credentials. Sources without a separate credential policy apply `rate` to both modes. `keyed_rate` changes a separately declared credential rate. Each rate is an object with positive whole-number `requests` and `seconds`, each within the unsigned 16-bit range. Requests are evenly spaced across that interval. `in_flight` is a positive whole number within the source's concurrency ceiling. Omitted fields retain their defaults.

```json
{
  "source_pacing": {
    "ncbi-eutils": {
      "rate": {"requests": 1, "seconds": 2},
      "keyed_rate": {"requests": 5, "seconds": 1},
      "in_flight": 1
    }
  }
}
```

The matching global flag accepts the same JSON map:

```sh
biomcp serve --hosted --source_pacing '{"ncbi-eutils":{"rate":{"requests":1,"seconds":1}}}'
```

Flag fields override file fields for that source. Other fields and sources remain as configured. Changing `rate` leaves a separate `keyed_rate` unchanged. Unknown sources, zero components and values above a source ceiling cause a named configuration error before source requests. Credential rates remain subject to `--rate-share` in hosted mode. Provider daily quotas, IP-wide limits and shared upstream pools still apply; this setting grants no extra provider capacity.

## Command deadlines and drug approvals

`command_timeout_seconds` sets a positive whole-second command budget, using the [shared command default](../crates/biomcp/src/config.rs) when omitted. A first bundle reader waits for an active synchronization within that budget and the source's deadline. Missing bundles still offer an explicit sync command; existing snapshots remain readable during replacement.

`drug_approvals_limit` sets the number of Drugs@FDA submissions returned by the approvals section, from 1 through 50. Omit it to use the [built-in submission page size](../crates/biomcp-types/src/drug/mod.rs). An explicit `--section-limit` overrides this setting. Continue with the returned page action when more submissions remain.

## MCP reply form

`mcp_reply_form` accepts `both` (JSON text plus `structuredContent`), `structured-only` (JSON in `structuredContent` plus a one-line text note), or `text-only` (JSON text). It applies to all MCP tools over stdio and HTTP, including error replies. The default is `both` for local and hosted servers. Choose `structured-only` only for clients that read `structuredContent`. When included, the output schema describes that JSON value regardless of reply form.

## Hosted servers

Use `biomcp serve --hosted` for a shared server, with `--http` for HTTP transport. Hosted mode refuses MCP file saves, `health`, `sync`, `store clean`, `--fresh` and `key` commands on both transports. Each refusal returns `usage.mcp_unavailable` and names hosted mode. Read-only queries keep the shared request cache and query pools. Public replies stay shared across callers. Replies from no-fallback sources remain available only to callers with the same key, including recorded replies and stored continuations. Hosted mode refuses older licensed recordings and stored continuations that lack caller ownership. The HTTP `GET /health` endpoint remains available for local server checks. Hosted HTTP calls may supply source credentials through the header names reported by `biomcp key list`. The list also reports each source's fallback tier. Caller headers override server credentials for that request only. Sources in the no-fallback tier require the caller's credential even when the server has a configured key. Other sources may use the server credential when the caller supplies none. Missing required credentials direct callers to their account settings. Tool arguments never accept keys. Ordinary HTTP mode ignores these headers.

Hosted HTTP requests accept `x-biomcp-use: restricted`, `commercial` or `academic`. Missing headers select `restricted`. Restricted use admits open and attribution sources and commercially permitted records from per-record sources. Sources without explicit noncommercial terms remain enabled. Caller keys never widen restricted use. Commercial use also admits a noncommercial or licensed source only with its required caller key; optional rate-limit keys never enable blocked sources. Academic use retains every configured source. Server exclusions always apply, including for cached and stored replies. Invalid declarations are refused before source access. Ordinary HTTP and stdio ignore the header.

The JSON setting `hosted_max_use` selects the highest hosted HTTP mode (`restricted`, `commercial` or `academic`, default `academic`). Deployments serving only commercially safe sources set `"hosted_max_use": "restricted"`. Higher caller declarations are refused with an error naming the allowed modes. This setting applies only to hosted HTTP; local CLI and stdio, including hosted stdio, keep every configured source. Hosted command and file restrictions still apply on both server transports.

The hosting front end must strip the client-sent `x-biomcp-use` header and set it from the authenticated account attestation. It must also strip all client-sent `x-biomcp-key-` headers before injecting credentials from the authenticated caller's account settings. BioMCP does not authenticate these headers itself.

Hosted source pacing follows each effective key. Keyless calls share one bucket per source. Set `--rate-share` to the fraction of each keyed source rate assigned to this server, such as `--rate-share 0.25` for each of four instances. Keyless hosted buckets and ordinary process buckets keep the source's full rate. The front end chooses these shares; servers do not coordinate rates with each other. Hosted HTTP calls have a whole-call wall-clock deadline and return the typed `deadline-exceeded` reason when it expires. The server keeps a bounded number of active calls, including timed-out work until it stops.

In hosted `restricted` mode, [cBioPortal study flags](cbioportal-study-terms.md) also withhold studies with noncommercial terms or a reviewed purpose limit when reading the existing catalog, saved study pages and recurrence observations.

`GET /health` checks that the local cache/store opens with compatible metadata within two seconds. It makes no upstream requests, does not verify every stored table or row, and does not promise that sources are available or a query will succeed. Incompatible metadata or an unreadable store returns 503. A missing or damaged payload table can still return 200; this endpoint does not repair it.

HTTP tool calls write one JSON diagnostic with the registered tool name, duration in milliseconds, reply status and failed source names. Arguments, query text and credentials are excluded.
