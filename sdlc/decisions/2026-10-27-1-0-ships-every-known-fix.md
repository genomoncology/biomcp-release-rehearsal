# 1.0 ships every known fix

Date: 2026-10-27

Decided by Ian: "My goal is to get everything done for 1.0. That we know about, at least."

- Every known defect, gap and polish item goes into 1.0: both docs findings reports, the items split off from release tickets, the security screening work, the build and check hygiene tickets, and AlphaGenome (0337, then 0198).
- New capabilities stay on the later list: the wiki, a Postgres store, study charts, bindings, the biodata merge, PDF supplement reading and a protein-to-rsID lookup. They are new features, not known defects.
- The release candidate is cut from main once every 1.0 ticket lands, instead of cherry-picking chosen commits onto `release/1.0`. `release/1.0` takes main's tree in one merge, so the public append still has the candidate as its second parent.
- Flattening deep result paths waits for 1.x. Amended 2026-11-01: Ian ruled no freeze and no compatibility promise (`2026-11-01-no-backwards-compatibility.md`), so the reason is size: it changes every reply schema, the docs, the skill guide and the recipes, and fixes no defect.

This replaces the 2026-10-27 practice of landing release fixes on main and cherry-picking them onto `release/1.0`, and the plan's line that AlphaGenome joins 1.0 only if ready.
