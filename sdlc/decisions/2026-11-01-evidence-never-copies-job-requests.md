# Evidence never copies job request files

Date: 2026-11-01. Owner: the PM.

Job request files hold private execution metadata, and pm creates them readable only by their owner. BioMCP evidence retention never copies a job request file or any other pm control file. Retained evidence names what it keeps: the job's log and the outputs the job declares. A script, delegate or manual retention step that selects files by globbing a job folder must exclude pm control files.

This replaces the 0535 rule that allowed a copy written with mode 600. Copies already made stay mode 600 until their ticket's evidence is cleaned up. A new copy is a defect, fixed by deleting it and narrowing the retention step that made it.
