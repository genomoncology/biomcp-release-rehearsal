# Release rehearsals run in the public rehearsal repository

Ian, 2026-11-02: "just use public rehearsals no cost".

GitHub refused the private rehearsal run 41400311764 for account billing. Release rehearsals now run in the public repository genomoncology/biomcp-release-rehearsal, where hosted runners cost nothing. Each run dispatches the release workflow with `rehearsal=true` from an exact landed main commit, under a non-`v` tag, with no credentials or secrets configured, so every publication and channel authentication step stays a no-op. Public runs attest their artifacts, which the private repository could not.

The rehearsal repository shows the candidate source publicly before the history append to the public BioMCP repository. That is accepted by this decision. The append, the 1.0 tag and publication still need Ian's approval.

The private repository genomoncology/biomcp-1-0-release-rehearsal is no longer used. Ian deletes it.
