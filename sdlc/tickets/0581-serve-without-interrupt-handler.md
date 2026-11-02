# Serve keeps running or says why when it cannot catch interrupts

Status: COMPLETE.
Landed: 50407d21070635e84cbc28f905e646ba8cdc1b56
Milestone: 5

## Outcome

`biomcp serve --hosted --http ADDR` started with SIGINT ignored, as under `nohup` or as an `sh` background job, either serves normally or exits with a one-line reason on stderr. It never exits silently.

- Why: the hosted team found it exits with code 5 and prints nothing in that case (their jobs j23707 and j23714 on M5, m20559). The likely cause is `crates/biomcp/src/mcp/http.rs` mapping a `ctrlc::try_set_handler` failure to `StartError::Local` with no message. Their ECS launch uses default signals, so it is not a launch blocker, but a silent exit costs every operator time.
- Starts from: `crates/biomcp/src/mcp/http.rs` and the stdio serve path if it shares the handler setup.
- Rules: prefer to keep serving when the handler cannot be set, since an ignored SIGINT means the parent chose that; shutdown by SIGTERM still works. If serving cannot continue, print one plain reason. No new flag.
- Proof: an installed-binary case starts serve with SIGINT ignored and gets an answer to an MCP request, or the reason line and a nonzero exit.
- Changes: the serve start path and one case.
- Defers: nothing.
