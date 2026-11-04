# Qualification job time limit fits the recipe

Status: COMPLETE.
Landed: cffe8afecd53518e1fbf581acf70c9047209c226
Milestone: 5

## Outcome

The release workflow's `qualify` job allows 120 minutes instead of 90, so the slowest platforms finish every installed round, the container check and the schema release check inside the limit.

- Why: private rehearsal run 41388677743 at dac98145 cancelled the Intel Mac and Linux x64 qualification jobs at "exceeded maximum execution time 1h30m0s" (`.github/workflows/release.yml:77`; annotations on Yellow `biomcp-redux-evidence/0294/private-rehearsal-41388677743/macos-intel-annotations.json` and `linux-x64-annotations.json`). Neither had a product failure: the Intel Mac had passed three installed rounds 527/0 and Linux x64 five installed rounds plus the container check, but both lost the remaining steps, including the schema release check the interim baseline ticket needs to prove. Speeding up the build is real work that 1.0 does not need before proof; a 30-minute raise is the smallest change that lets the current recipe finish.
- Starts from: `.github/workflows/release.yml` (`qualify`), `release/test_workflow_contracts.py` or whichever static test pins workflow limits.
- Rules: change only the `qualify` limit to 120 minutes; keep a bound on every job; add no retries, services or steps. If a static test pins the limit, update it to the new value.
- Proof: the cheap checks pass; the next private rehearsal's Intel Mac and Linux x64 qualification jobs finish inside the limit, and the run records each job's duration.
- Changes: `release.yml` and its static test only.
- Defers: build and test speed work, which waits until after 1.0 unless 120 minutes also proves too short.
