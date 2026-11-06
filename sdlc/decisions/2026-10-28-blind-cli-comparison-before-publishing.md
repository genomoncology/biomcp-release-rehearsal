# A blind CLI comparison is the last step before publishing 1.0

Date: 2026-10-28

Decided by Ian.

Before 1.0 is published, a fresh agent that knows nothing about this project compares the released 0.9.3 CLI with the 1.0 candidate CLI on the same set of about 20 tasks. Ian wants "a very thorough analysis to make sure that there's no backpedaling in quality or results or anything in 1.0, and that anything that is different is different for a reason."

- The agent works in an empty folder with only the two installed binaries and the task list. It reads each binary's own help and has no repository, docs site or prior notes.
- The comparison covers ease of use, the tokens each answer costs, chaining commands together, use with `jq`, getting at the underlying data, and whether each task's question is answered correctly.
- Every case where 1.0 does worse is either fixed before publication or explained by a recorded ruling.
- Ian is told when the final candidate is ready for this comparison. Publication waits for its result and Ian's approval.

Update, 2026-11-06: Ian moved this comparison to evals and dropped the dependency (m23337, m23350). It no longer gates publishing.
