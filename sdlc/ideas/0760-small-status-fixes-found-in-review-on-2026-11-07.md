# Small status fixes found in review on 2026-11-07: (1) MyChem get-by-name rebuilds status as partial when unsettled even if the selected get failed (resolve.rs end of get; guard with status == Complete), with cases for a product-only answer after the cap (partial) and after reading every row (complete); (2) an over-limit reply is reported as 'bad reply' instead of 'too large' in the fetch layer.

Status: closed.
Promoted: 0780, 0784

Kind: idea
When: after the 0640 preflight, before the 1.0 cut

## Problem

Small status fixes found in review on 2026-11-07: (1) MyChem get-by-name rebuilds status as partial when unsettled even if the selected get failed (resolve.rs end of get; guard with status == Complete), with cases for a product-only answer after the cap (partial) and after reading every row (complete); (2) an over-limit reply is reported as 'bad reply' instead of 'too large' in the fetch layer. status and exit codes that say what really happened.
