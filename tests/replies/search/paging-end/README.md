# Paging end

These replies test where a next page stops. Trial IDs, sponsors, totals and page tokens are synthetic controls. The trial row reuses the shape of the retained ClinicalTrials.gov capture in `trial/country`, and the PMA rows reuse the public FDA rows in `diagnostic/device-approvals`. The registry sends a page token on a full last page, as that capture shows. The registry gives a total only on a first page; on a later page the product asks a separate count request, and the failed count reply stands in for one that cannot answer.
