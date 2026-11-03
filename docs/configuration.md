# Server configuration

BioMCP reads the configuration file described in the README. Unknown keys cause a configuration error. Command flags override their configuration keys.

## Cache and store folders

On Unix, BioMCP keeps its cache and stored collections in `$XDG_CACHE_HOME/biomcp`, or `~/.cache/biomcp` when `XDG_CACHE_HOME` is unset. On Windows, it uses `%LOCALAPPDATA%/biomcp`. Choose a local disk for these folders. Sharing one cache between several processes through a Docker Desktop host bind mount or a network filesystem is unsupported. BioMCP uses SQLite's write-ahead log (WAL), which needs shared memory between processes on the same host; see the [storage caution](cautions.md#cache-and-store-storage).

### Docker

Mount a Docker named volume at the container's `XDG_CACHE_HOME` path to keep the cache and stored collections on container-native storage. The published image uses `/tmp/cache`. Use a volume writable by the image's user, UID 65532. Sharing one cache between several processes through a Docker Desktop host bind mount or a network filesystem is unsupported; use a local named volume instead.

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
