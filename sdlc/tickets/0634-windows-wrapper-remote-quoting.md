# Windows release wrapper runs under default PowerShell

Status: COMPLETE.
Landed: fd829000c1f44cead03d95a9c7dcbb98709c37d2
Milestone: 5

## Outcome

`scripts/check_windows_release.sh` builds every remote command (make directory, change directory, cleanup) through one safe construction that the Windows VM's default PowerShell passes to `cmd` unchanged, so the real wrapper transfers the release and runs the gate.

- Why: m25326. Wrapper job j29782 at 314dbebb failed before transfer or any product step: line 38 sends `cmd /d /c if exist ... (exit /b 1) else (mkdir ...)` unquoted, and default PowerShell parses the parenthesized payload ("exit" not recognized). Hosted Windows qualification passed, but 0293 needs the real wrapper proof (0621).
- Starts from: `scripts/check_windows_release.sh` and its existing tests.
- Rules: proof tooling only; the frozen product ed9c36a96 and the candidate are unchanged. One helper for all same-shaped remote calls, not a fix at line 38 alone. No VM, sandbox or account setting change.
- Proof: a quoting regression runs the helper's commands through default PowerShell with a path containing spaces and parentheses; then the real VM wrapper transfers the candidate and runs the gate.
- Changes: `scripts/check_windows_release.sh`, `scripts/test_*.py`; the remote command helper, its call sites and the regression.
- Defers: nothing.
