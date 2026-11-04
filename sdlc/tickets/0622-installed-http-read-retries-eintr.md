# Installed HTTP deadline case retries an interrupted read

Status: COMPLETE.
Landed: 5afd004f16c24d57be61c8f7bbd60e8278cd6fc0
Milestone: 5

## Outcome

The installed `mcp-http/framing-resource-deadlines-and-shutdown` case retries a read that returns `Interrupted`, as the product server already does, so a signal under emulation no longer fails the release qualification, while a real early close or missed deadline still fails.

- Why: cold qualification j29497 at the final head ed9c36a96 on M5 Docker x86 emulation passed 599 of 600 installed cases. The failure was `read=Err(Interrupted)` at 38.897 ms in the slow-header deadline check (m25012). `crates/biomcp/tests/runner/http_limits.rs:101` makes one bare `read` and lines 109–113 accept only EOF or connection reset (m25013). The product loop at `crates/biomcp/src/mcp/http.rs:141` already continues on `Interrupted`, so this is a test assertion defect, not a product one.
- Starts from: `crates/biomcp/tests/runner/http_limits.rs` and any other bare `read` in `crates/biomcp/tests/runner/` that asserts on its first result.
- Rules: test runner only; the frozen product source is unchanged and the cut is not reopened. Retry only on `Interrupted`, keeping the elapsed-time measurement from the first attempt so a deadline is still checked against the server budget. Fix every bare read in the runner with the same shape through one helper, not this case alone.
- Proof: a unit test injects an `Interrupted` first read and the helper continues to EOF; the focused installed case passes under the same M5 x86 emulation; a server that closes early still fails the case.
- Changes: `crates/biomcp/tests/runner/http_limits.rs`, `crates/biomcp/tests/runner/**`; the read helper, its callers and a test.
- Defers: nothing.
