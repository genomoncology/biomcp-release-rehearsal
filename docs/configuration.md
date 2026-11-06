# Server configuration

BioMCP reads the configuration file described in the README. Unknown keys cause a configuration error. Command flags override their configuration keys.

## Cache and store folders

On Unix, BioMCP keeps its cache and stored collections in `$XDG_CACHE_HOME/biomcp`, or `~/.cache/biomcp` when `XDG_CACHE_HOME` is unset. On Windows, it uses `%LOCALAPPDATA%/biomcp`. Choose a local disk for these folders. Sharing one cache between several processes through a Docker Desktop host bind mount or a network filesystem is unsupported. BioMCP uses SQLite's write-ahead log (WAL), which needs shared memory between processes on the same host; see the [storage caution](cautions.md#cache-and-store-storage).

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

`http_max_calls` limits tool calls in flight. `http_max_connections` limits open non-health HTTP connections. Each key accepts a positive integer. The server refuses `http_max_connections` when it is below `http_max_calls`. Pending framing readers and active requests share the connection cap. Framing readers follow the connection setting and reserve a reader to classify health checks. `/health` uses neither the connection cap nor the tool call cap.

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

Hosted HTTP servers can set `"hosted_max_use": "restricted"` in JSON configuration to allow only `restricted` declarations, even when callers supply source keys. The setting accepts `restricted`, `commercial` or `academic` and defaults to `academic`. Requests above the ceiling are refused with an error naming the allowed modes; missing headers still select `restricted`. Ordinary CLI and stdio are unchanged.
