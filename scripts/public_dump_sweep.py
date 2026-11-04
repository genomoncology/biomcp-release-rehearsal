"""Sweep pinned local public releases through BioData; never download data.

Build the test bridge with cargo build -p biomcp-types --example public_dump_values.
Pass a JSON manifest with sources: [{kind, path, release, url, rights, sha256}].
Paths resolve against the manifest directory and dumps must remain outside Git.
Kinds: civic_variants, civic_profiles, civic_evidence, clinvar_summary,
clingen_summary, clingen_evidence, clinvar_gks. ClinGen evidence accepts one
original public interpretation response per file; GKS accepts the keyed bundle.
The clinvar_gks_record kind accepts one original publisher example record.
Only CC0-1.0 and public-domain inputs are accepted. Pin the compressed file hash.
This tool checks mapped source fields. Monthly CIViC profiles require a native
rawName companion: profile_companions [{path,sha256,url,retrieved,source_sha256}].
Each original GraphQL body contains data.molecularProfiles.nodes with id, name,
rawName and variants [{id,name,feature:{id,name}}]. source_sha256 must be the
sorted exact hashes of all target profile and variant TSVs. API retrieval remains
separate from the monthly release. Members and feature identities must match
those TSVs exactly. Only Rust parses rawName; display names are never grammar.
Missing/ambiguous companions, extra companion IDs and inconsistent members fail by profile ID and
cannot be waived. --output retains every profile refusal; stdout is bounded.
Partial runs mark the companion comparison incomplete and do not infer that
companion IDs are absent from unvisited monthly rows.
Use --observed with saved CLI envelopes as JSONL: {source, release, revision,
retrieved, source_sha256, record, reply}. The manifest must name product_revision.
The observation revision and sorted source_sha256 list must match these inputs. Source is civic or clinvar; ClinVar record is its
clinvar: Variation ID. Release must exactly match the source manifest. Civic
replies must cover every source gene. --values-only explicitly omits comparison.
For CIViC, each observation also requires gene_variants companions, with complete
native variants and molecularProfiles pages, profile accepted evidence counts,
and complete assertion status pages. public_dump_civic_scope documents the saved
fields. The raw CLI count must match the captured GeneVariants total. Comparison
uses only direct gene variants with accepted evidence or assertions. --output
retains both raw totals and every raw/accepted unmatched ID; stdout reports each
omitted list length explicitly.
Each source may provide refusal_explanations entries with exact record, type,
value, reason, and source_rule. The source_rule names the publisher's invalid-input
rule and its URL. Unmatched refusals remain unexplained and fail the run.
Default output is bounded by SOURCE_LIMIT, EXAMPLE_LIMIT and GENE_LIMIT;
--output retains the full source inventory and counts.
Exit 0 means complete with no refusals or mapping gaps; 1 means findings or a
bounded partial run; 2 means an input, transport or tool failure.
"""
import argparse
from collections import Counter, defaultdict
import gzip
import json
from pathlib import Path
import subprocess
import sys

from public_dump_formats import REQUIRED, records
from public_dump_compare import Comparison
from public_dump_profiles import ProfileCompanions, pinned_file

EXAMPLE_LIMIT = 10
GENE_LIMIT = 20
SOURCE_LIMIT = 20
PIN_FIELDS = ("kind", "path", "release", "url", "rights", "sha256")


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--values", type=Path, required=True, help="built public_dump_values bridge")
    parser.add_argument("--max-records", type=int, help="bound records per file; marks report incomplete")
    parser.add_argument("--output", type=Path, help="save full counts and every unmatched ID; stdout is bounded")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--observed", type=Path, help="saved real CLI replies with source/release identity")
    mode.add_argument("--values-only", action="store_true", help="parse values without claiming reply equality")
    return parser.parse_args()


def pinned_sources(manifest):
    sources = json.loads(manifest.read_text())["sources"]
    if not isinstance(sources, list) or not sources:
        raise ValueError("manifest_empty_sources")
    for source in sources:
        required = set(PIN_FIELDS)
        if not required <= source.keys() or any(not source[key] for key in required):
            raise ValueError("manifest_missing_pin")
        if source["kind"] not in {*REQUIRED, "clinvar_gks", "clinvar_gks_record", "clingen_evidence"}:
            raise ValueError("manifest_unknown_source_kind")
        if source["rights"] not in {"CC0-1.0", "public-domain"}:
            raise ValueError("manifest_unapproved_rights")
        yield source, pinned_file(manifest, source)


class Sweep:
    def __init__(self, bridge, profiles):
        self.bridge = bridge
        self.profiles = profiles
        self.profile_refusals = []
        self.types = defaultdict(Counter)
        self.reasons = Counter()
        self.refusals = []
        self.genes = Counter()
        self.unmapped = Counter()
        self.sources = []
        self.spelling_mismatches = 0
        self.complete = True
        self.comparison = Comparison()
        self.unexplained = 0

    def parse(self, values, source, identity):
        mapped = [value for value in values if "mapping_reason" not in value]
        self.bridge.stdin.write(json.dumps(mapped) + "\n")
        self.bridge.stdin.flush()
        line = self.bridge.stdout.readline()
        if not line:
            raise ValueError("value_bridge_closed")
        outputs = json.loads(line)
        if len(outputs) != len(mapped):
            raise ValueError("value_bridge_count_mismatch")
        results = iter(outputs)
        refused = False
        for original in values:
            result = ({"kind": original["kind"], "value": original["value"],
                       "reason": original["mapping_reason"]} if "mapping_reason" in original else next(results))
            if result["kind"] != original["kind"] or result["value"] != original["value"]:
                raise ValueError("value_bridge_identity_mismatch")
            kind = result["kind"]
            reason = result.get("reason")
            if not reason and kind in {"variant", "clinvar_variation", "clinvar_allele", "vcv", "rcv", "scv"}:
                if result["parsed"] != result["value"]:
                    reason = "source_identifier_spelling_changed"
                    self.spelling_mismatches += 1
            self.types[kind]["refused" if reason else "parsed"] += 1
            if reason:
                refused = True
                if kind == "profile":
                    self.profile_refusals.append({"profile_id": identity, "reason": reason})
                self.reasons[f"{kind}: {reason}"] += 1
                rule = next((entry["source_rule"] for entry in source.get("refusal_explanations", [])
                             if entry.get("record") == identity and entry.get("type") == kind
                             and entry.get("value") == result["value"] and entry.get("reason") == reason
                             and entry.get("source_rule") and "mapping_reason" not in original
                             and reason != "source_identifier_spelling_changed" and kind != "profile"), None)
                self.types[kind]["explained" if rule else "unexplained"] += 1
                self.unexplained += not bool(rule)
                if len(self.refusals) < EXAMPLE_LIMIT:
                    self.refusals.append({"source": source["kind"], "record": identity, "type": kind,
                                          "value": result["value"], "reason": reason, "source_rule": rule})
        return refused

    def read(self, source, path, maximum):
        count = refused = empty = 0
        opener = gzip.open if path.suffix == ".gz" else open
        with opener(path, "rt", encoding="utf-8", newline="") as handle:
            for identity, values, genes, unmapped, context in records(handle, source["kind"]):
                if maximum is not None and count >= maximum:
                    self.complete = False
                    break
                count += 1
                if source["kind"] == "civic_profiles":
                    values[0] = self.profiles.resolve(values[0], source["release"])
                refused += self.parse(values, source, identity)
                empty += not values
                self.genes.update(genes)
                self.comparison.add(context, source["kind"], source["release"])
                if unmapped:
                    self.unmapped[unmapped] += 1
        pins = {key: source[key] for key in PIN_FIELDS}
        self.sources.append({**pins, "records": count, "parsed_records": count - refused - empty,
                             "refused_records": refused, "records_without_values": empty})

    def report(self):
        return {"complete": self.complete, "sources": self.sources,
                "profile_refusals": self.profile_refusals,
                "profile_companion_comparison_complete": self.complete,
                "profile_companion_provenance": self.profiles.provenance,
                "types": {kind: {key: counts[key] for key in ("parsed", "refused", "explained", "unexplained")}
                          for kind, counts in sorted(self.types.items())},
                "refusal_reasons": dict(self.reasons), "refusals": self.refusals,
                "gene_counts": dict(sorted(self.genes.items())),
                "unmapped_sections": dict(self.unmapped),
                "spelling_mismatches": self.spelling_mismatches}


def run(args):
    if args.max_records is not None and args.max_records <= 0:
        raise ValueError("max_records_must_be_positive")
    # Validate every file before starting the bridge or reporting partial success.
    sources = list(pinned_sources(args.manifest.resolve()))
    with ProfileCompanions(args.manifest.resolve(), sources, args.max_records) as profiles, \
            subprocess.Popen([str(args.values.resolve())], stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, text=True) as bridge:
        try:
            sweep = Sweep(bridge, profiles)
            for source, path in sources:
                sweep.read(source, path, args.max_records)
            if sweep.complete:
                for (identity,) in profiles.unmatched():
                    sweep.parse([{"kind": "profile", "value": "civic:" + identity,
                                  "mapping_reason": "profile_companion_not_in_monthly_input"}],
                                {"kind": "civic_profiles"}, "civic:" + identity)
            bridge.stdin.close()
            if bridge.wait() != 0:
                raise ValueError("value_bridge_failed")
        except BaseException:
            bridge.kill()
            bridge.wait()
            raise
    report = sweep.report()
    try:
        report["comparisons"] = sweep.comparison.compare(
            args.observed, sweep.genes, json.loads(args.manifest.read_text()).get("product_revision"),
            sources) if args.observed else None
    finally:
        sweep.comparison.close()
    if args.output:
        with args.output.open("x", encoding="utf-8") as output:
            json.dump(report, output, indent=2, sort_keys=True)
            output.write("\n")
    if report["comparisons"]:
        provenance = report["comparisons"]["provenance"]
        report["comparisons"]["provenance_omitted"] = max(0, len(provenance) - EXAMPLE_LIMIT)
        report["comparisons"]["provenance"] = provenance[:EXAMPLE_LIMIT]
        rows = report["comparisons"]["civic_gene_comparisons"]
        report["comparisons"]["civic_gene_comparisons_omitted"] = max(0, len(rows) - GENE_LIMIT)
        report["comparisons"]["civic_gene_comparisons"] = rows[:GENE_LIMIT]
        for row in report["comparisons"]["civic_gene_comparisons"]:
            for key, values in list(row.items()):
                if key.endswith("_ids"):
                    row[key + "_omitted"] = max(0, len(values) - GENE_LIMIT)
                    row[key] = values[:GENE_LIMIT]
    for key in ("profile_refusals", "profile_companion_provenance"):
        report[key + "_omitted"] = max(0, len(report[key]) - EXAMPLE_LIMIT)
        report[key] = report[key][:EXAMPLE_LIMIT]
    report["sources_omitted"] = max(0, len(report["sources"]) - SOURCE_LIMIT)
    report["sources"] = report["sources"][:SOURCE_LIMIT]
    report["gene_counts_omitted"] = max(0, len(report["gene_counts"]) - GENE_LIMIT)
    report["gene_counts"] = dict(list(report["gene_counts"].items())[:GENE_LIMIT])
    reasons = report["refusal_reasons"]
    report["refusal_reasons_omitted"] = max(0, len(reasons) - EXAMPLE_LIMIT)
    report["refusal_reasons"] = {reason[:512]: count for reason, count in list(reasons.items())[:EXAMPLE_LIMIT]}
    report["refusals"] = [{**refusal, "value": str(refusal["value"])[:512],
                           "reason": refusal["reason"][:512]} for refusal in report["refusals"]]
    print(json.dumps(report, indent=2, sort_keys=True))
    comparisons_failed = report["comparisons"] and report["comparisons"]["failures"]
    failures = sweep.unexplained or sweep.unmapped or comparisons_failed or any(row["records_without_values"] for row in sweep.sources)
    return int(bool(failures) or not sweep.complete)


def main():
    try:
        return run(arguments())
    except (OSError, ValueError, KeyError, TypeError, BrokenPipeError) as error:
        print(f"public_dump_sweep: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
