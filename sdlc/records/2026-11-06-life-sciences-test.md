# Life sciences test of the hosted server, 2026-11-06

Ian had an agent run about 40 calls across eight pharma workflows on the hosted server at deployed revision 1bb97e64, and spot-check results against the web. Some findings may already be fixed on main; each ticket reproduces first.

## Verdict

| Workflow | Verdict |
|---|---|
| Competitive intelligence from trials | Strong |
| Trial results | Strong |
| Label and safety review | Strong |
| Regulatory: EMA, recalls, devices | Good |
| Target validation | Good |
| Literature monitoring | Good |
| Finding key opinion leaders | Broken |
| Epidemiology from SEER | Broken |
| Mutation frequency in a cohort | Broken |
| Biomarkers and NCI trials | Blocked on a user API key |

The report's bottom line: the biggest risk is silent incompleteness, from synonym gaps and stale sources that give answers which look complete but are not.

## Findings to tickets

| Finding | Ticket |
|---|---|
| Item 1, no synonym expansion | 0717 |
| Items 2 and 10, stale data with no flag; `sync` offered on hosted | 0718 |
| Item 3, trial lists hide interventions | 0719 |
| Item 4, SEER unusable | 0720 |
| Mutation frequency rejected every form | 0721 |
| Finding key opinion leaders | 0722 |
| Items 5 and 6, schema and filter mismatches | 0723 |
| Item 8, raw source formatting | 0724 |
| Item 11, empty next page | 0725 |
| Item 12, cell line search | 0726 |
| Biomarkers and NCI blocked; Ian's per-user keys and sources | 0727, with hosting |

The report numbers its items 1 to 6, 8, 10, 11 and 12; there is no 7 or 9.
