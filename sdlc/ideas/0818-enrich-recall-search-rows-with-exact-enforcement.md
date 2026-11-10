# Enrich recall search rows with exact enforcement classification

Status: open.

Kind: idea
When: When recall search enrichment is scheduled for 1.1

## Problem

Enrich recall search rows with exact enforcement classification. ClassII evidence comes from device/enforcement, not device/recall RES search. Existing detail already exact-joins recall_number. Search enrichment needs bounded exact joins, absent/invalid/mismatched-identity controls and visible enrichment failures without changing native search totals/paging; never infer classification from status or claim source feed repair. Classification filtering is separate scope.
