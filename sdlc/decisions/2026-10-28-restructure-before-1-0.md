# The code quality restructuring ships in 1.0

Date: 2026-10-28

Decided by Ian ("No, let's get it all done. I want to do the restructuring now.").

Every finding of `records/2026-10-28-code-quality-review.md` lands before 1.0, including the structure work that changes no user-visible behavior: shared status and typed next steps (0419), query behavior in the query layer (0420), one command table (0421), one source declaration (0422), the AlphaGenome adapter codec (0423), shared helpers (0424), typed errors (0426), one transport request path (0428) and the architecture rewrite (0432). The PM's recommendation to defer them to 1.0.x was declined, and the release date moves by about one to two weeks.

The coordinator orders the work to avoid collisions with the benchmark tickets 0408 to 0416, and each ticket keeps existing cases passing.
