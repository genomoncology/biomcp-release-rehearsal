# Build plan for the remaining product issues

The current plan is [../plan.md](../plan.md). This file orders implementation by harm and overlapping owners; it is not a ticket-status ledger. The [November 10 review](../records/2026-11-10-product-review.md) and [test playbook](remaining-issues-playbook.md) supply source evidence and exact acceptance scenarios.

## First wave

Run independent lanes for GWAS threshold uncertainty, complete drug-name resolution, trial native pagination and hosting signing-key freshness. These fix incorrect claims or a security boundary. The two fetch-cache fixes can share one lane and one fixture review because they touch the same cache lookup and offline paths. Nightly diagnostics and source-policy documentation are independent small lanes.

## Shared pool owner

The source-snapshot fix touches several entity callers, including the GWAS threshold file. Design its eligibility interface first, then stack it after the GWAS change or assign both to one lane; do not let two writers independently change that file. Preserve whole-answer eligibility under concurrent extension and explicit old-pool reads. Avoid changing raw-response cache policy without a separate demonstrated defect.

Drug-name expansion and trial pagination both use trial fixtures but have different production owners. Keep fixture edits separate or stack them if the same case file is needed. Documentation integrates after the behavior it explains is fixed. Hosting remains an independent repository and can land its local fix without deploying.

## Acceptance and batching

Each fix starts with its decisive synthetic/public-boundary case, receives one fresh review, and joins a coherent batch. One normal landing gate checks the combined candidate. The fresh review of this ticket set checks architecture and edge cases before handover. New observations that remain uncertain stay in the playbook with a stop rule; do not turn every test opportunity into a new framework or one ticket per assertion.

M5 is the ordinary build host. Yellow is permitted by Ian but its November 10 probe stopped because its configured compiler wrapper could not find sccache; fix or bypass that specific setup before another Yellow compile. Do not treat it as a product failure. Tests use pm job, pinned Rust 1.95.0 and Python 3.13 where applicable. Existing artifacts can be reused for behavioral probing when their source and hashes are verified.

## After implementation

Select the ending source, refresh only affected qualification, then finish the existing manual installed checks. Refresh and independently review the history-append preparation before assembling the publication proposal. Hosted validation and actual publication retain their existing approvals; release timing is not a reason to waive an accepted correctness issue. Keep prior failures, nonblocking mutation rationales, migration guidance and real platform labels in the final package.
