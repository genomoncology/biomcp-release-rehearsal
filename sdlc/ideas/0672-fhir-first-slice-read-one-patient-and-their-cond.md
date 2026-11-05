# FHIR first slice: read one patient and their conditions from one FHIR server, read-only, and follow typed links into BioMCP genes, variants, trials and drugs. Rules from Ian's note (2026-10-27): patient data never enters fixtures, logs, caches or Git (tests use Synthea, the MIMIC-IV FHIR demo and HL7 examples); hosted HTTP refuses patient reads; SMART sign-in keys stay in the environment; vendor quirks recorded with source in one place; BioData patient types exist first. Old BioMCP built this slice once; the PicoHR FHIRPath set and HL7 example resources are on disk. Source: Jarvis m27028.

Status: open.

Kind: idea
When: after 1.0 publishes

## Problem

FHIR first slice: read one patient and their conditions from one FHIR server, read-only, and follow typed links into BioMCP genes, variants, trials and drugs. Rules from Ian's note (2026-10-27): patient data never enters fixtures, logs, caches or Git (tests use Synthea, the MIMIC-IV FHIR demo and HL7 examples); hosted HTTP refuses patient reads; SMART sign-in keys stay in the environment; vendor quirks recorded with source in one place; BioData patient types exist first. Old BioMCP built this slice once; the PicoHR FHIRPath set and HL7 example resources are on disk. Source: Jarvis m27028. the bridge from the free research tool to the on-premises engine where patient data lives; first slice is one ticket.
