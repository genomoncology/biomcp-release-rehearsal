# BioData 1.0 publication approved

Ian, 2026-11-02: "I've approved bio data, and it's going to publish the public repo. I don't care about the 2% performance hit."

BioData publishes genomoncology/biodata with tag v1.0.0. The 2% catalog lookup slowdown another product measured does not block publication. BioMCP's switch (0459) repins to the published tag, removes the frozen copy and runs the final gate. If the published commit differs from 08d9399cc0a29d36fd48c002ccec57a34df37f09, BioMCP rechecks before it repins.
