# Server configuration

BioMCP reads the configuration file described in the README. Unknown keys cause a configuration error. Command flags override their configuration keys.

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
