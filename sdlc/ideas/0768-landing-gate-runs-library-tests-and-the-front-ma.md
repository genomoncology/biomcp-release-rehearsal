# Landing gate runs library tests and the front-map check for crates a change touches. Today 0766 passed its landing gate twice and then failed the release full suite twice: once on check_front_map.py (docs-check, 6 min) and once on biomcp-types drug_contracts (library-tests, 3 min). Neither stage runs in the landing gate.

Status: open.

Kind: idea
When: after the 1.0 cut

## Problem

Landing gate runs library tests and the front-map check for crates a change touches. Today 0766 passed its landing gate twice and then failed the release full suite twice: once on check_front_map.py (docs-check, 6 min) and once on biomcp-types drug_contracts (library-tests, 3 min). Neither stage runs in the landing gate. release finds no bugs: about 9 more minutes per landing that touches those crates instead of a reopened ticket and a full-suite rerun.
