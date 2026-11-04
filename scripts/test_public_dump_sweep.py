"""Exercise the offline sweep through its command and the real value bridge."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "scripts/public_dump_sweep.py"
BRIDGE = os.environ.get("PUBLIC_DUMP_VALUES")


@unittest.skipUnless(BRIDGE, "build public_dump_values and set PUBLIC_DUMP_VALUES")
class Sweep(unittest.TestCase):
    def run_sweep(self, kind, content, *, digest=None, limit=None, observed=None, explanations=None, full=False,
                  profile_capture=None, variant_content=None, capture_overrides=None):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            data = gzip.compress(content.encode(), mtime=0)
            (folder / "source.gz").write_bytes(data)
            manifest = {"product_revision": "synthetic-product-reply", "sources": [{
                "kind": kind, "path": "source.gz", "release": "synthetic-edge-sample",
                "url": "https://example.invalid/public-format-fixture",
                "rights": "CC0-1.0", "sha256": digest or hashlib.sha256(data).hexdigest(),
                "refusal_explanations": explanations or [],
            }]}
            if variant_content is not None:
                variant_data = gzip.compress(variant_content.encode(), mtime=0)
                (folder / "variants.gz").write_bytes(variant_data)
                manifest["sources"].append({**manifest["sources"][0], "kind": "civic_variants",
                                            "path": "variants.gz", "sha256": hashlib.sha256(variant_data).hexdigest()})
            if profile_capture is not None:
                captured = json.dumps(profile_capture).encode()
                (folder / "profiles.json").write_bytes(captured)
                manifest["profile_companions"] = [{
                    "path": "profiles.json", "sha256": hashlib.sha256(captured).hexdigest(),
                    "url": "https://example.invalid/synthetic-native-profile",
                    "retrieved": "2026-10-29", "source_sha256": sorted(source["sha256"] for source in manifest["sources"]),
                    **(capture_overrides or {}),
                }]
            (folder / "manifest.json").write_text(json.dumps(manifest))
            command = [sys.executable, str(TOOL), str(folder / "manifest.json"),
                       "--values", BRIDGE]
            if observed is None:
                command += ["--values-only"]
            else:
                for row in observed:
                    row.setdefault("source_sha256", [manifest["sources"][0]["sha256"]])
                (folder / "observed.jsonl").write_text("\n".join(json.dumps(row) for row in observed))
                command += ["--observed", str(folder / "observed.jsonl")]
            if limit is not None:
                command += ["--max-records", str(limit)]
            if full:
                command += ["--output", str(folder / "report.json")]
            result = subprocess.run(command, text=True, capture_output=True, check=False)
            report = json.loads(result.stdout) if result.stdout else None
            if full and report:
                report["full_report"] = json.loads((folder / "report.json").read_text())
            return result, report

    def test_counts_each_row_and_preserves_compound_profile_refusal(self):
        text = ("molecular_profile_id\tname\tvariant_ids\n"
                "9\tBRAF V600E AND NOT KRAS G12D\t12,13\n")
        result, report = self.run_sweep("civic_profiles", text)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["sources"][0]["records"], 1)
        self.assertEqual(report["types"]["profile"]["refused"], 1)
        self.assertEqual(report["types"]["identifier"]["parsed"], 2)

    def test_source_values_cannot_disappear_behind_successful_records(self):
        bundle = {"condition": {"MONDO:0004992": {"id": "MONDO:0004992", "label": "cancer"}},
                  "therapy": {"therapy:1": {"id": "therapy:1", "label": "drug"}}}
        result, report = self.run_sweep("clinvar_gks", json.dumps(bundle))
        self.assertEqual(result.returncode, 0, result.stderr)
        for kind in ("disease", "drug"):
            self.assertEqual(report["types"][kind]["parsed"], 1)
        text = "variant_id\tvariant\tgene\thgvs_descriptions\t5_prime_partner\t3_prime_partner\n1\tFusion\t\t\tBCR\tABL1\n"
        result, report = self.run_sweep("civic_variants", text)
        self.assertEqual(report["types"]["gene"]["parsed"], 2)

    def test_codings_and_native_accessions_keep_source_authority(self):
        record = {"id": "clinvar:12", "mappings": [{"coding": {
            "system": "https://www.genenames.org", "code": "6407"}}],
            "sequenceReference": {"refgetAccession": "SQ.source"}}
        result, report = self.run_sweep("clinvar_gks_record", json.dumps(record))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["types"]["identifier"]["parsed"], 1)
        self.assertEqual(report["types"]["native_identifier"]["parsed"], 1)

    def test_native_profile_members_do_not_become_empty_parsed_profiles(self):
        text = "molecular_profile_id\tname\tvariant_ids\n1\tBCR::ABL1 Fusion\t1\n"
        result, report = self.run_sweep("civic_profiles", text)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["types"]["profile"]["parsed"], 0)
        self.assertEqual(report["types"]["profile"]["unexplained"], 1)
        self.assertEqual(report["refusals"][0]["record"], "civic:1")
        self.assertEqual(report["refusals"][0]["reason"], "profile_companion_missing")

    def profile_inputs(self):
        label = "BRAF V600E AND NOT KRAS G12D"
        profiles = "molecular_profile_id\tname\tvariant_ids\n9\t" + label + "\t12,13\n"
        variants = ("variant_id\tvariant\tgene\thgvs_descriptions\tfeature_id\tfeature_name\n"
                    "12\tV600E\tBRAF\t\t5\tBRAF\n13\tG12D\tKRAS\t\t30\tKRAS\n")
        node = {"id": 9, "name": label, "rawName": "#VID12 AND NOT #VID13", "isComplex": True,
                "variants": [{"id": 12, "name": "V600E", "feature": {"id": 5, "name": "BRAF"}},
                             {"id": 13, "name": "G12D", "feature": {"id": 30, "name": "KRAS"}}]}
        return profiles, variants, {"data": {"molecularProfiles": {"nodes": [node]}}}

    def test_monthly_profile_uses_bound_native_expression_and_members(self):
        profiles, variants, capture = self.profile_inputs()
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture)
        self.assertEqual(result.returncode, 0, result.stderr + str(report))
        self.assertEqual(report["types"]["profile"]["parsed"], 1)
        provenance = report["profile_companion_provenance"][0]
        self.assertEqual(provenance["retrieved"], "2026-10-29")
        self.assertEqual(provenance["source_sha256"], sorted(source["sha256"] for source in report["sources"]))

    def test_profile_companion_ambiguity_and_member_mismatch_are_reported_by_id(self):
        profiles, variants, capture = self.profile_inputs()
        nodes = capture["data"]["molecularProfiles"]["nodes"]
        nodes.append(nodes[0].copy())
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"], [{"profile_id": "civic:9", "reason": "profile_companion_ambiguous"}])
        nodes.pop()
        nodes[0]["variants"].pop()
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"], [{"profile_id": "civic:9", "reason": "profile_member_ids_mismatch"}])

    def test_companion_only_profile_is_refused_by_id(self):
        profiles, variants, capture = self.profile_inputs()
        nodes = capture["data"]["molecularProfiles"]["nodes"]
        nodes.append({**nodes[0], "id": 10})
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants,
                                        profile_capture=capture, full=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["types"]["profile"]["parsed"], 1)
        self.assertEqual(report["types"]["profile"]["unexplained"], 1)
        self.assertEqual(report["full_report"]["profile_refusals"],
                         [{"profile_id": "civic:10", "reason": "profile_companion_not_in_monthly_input"}])
        self.assertTrue(report["profile_companion_comparison_complete"])
        profiles += "10\t" + nodes[0]["name"] + "\t12,13\n"
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants,
                                        profile_capture=capture, limit=1)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertFalse(report["complete"])
        self.assertFalse(report["profile_companion_comparison_complete"])
        self.assertNotIn("civic:10", [row["profile_id"] for row in report["profile_refusals"]])

    def test_profile_companion_requires_exact_provenance_and_feature_identity(self):
        profiles, variants, capture = self.profile_inputs()
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture,
                                        capture_overrides={"source_sha256": ["0" * 64]})
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"][0]["reason"], "profile_companion_source_hash_mismatch")
        capture["data"]["molecularProfiles"]["nodes"][0]["variants"][0]["feature"]["name"] = "OTHER"
        result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"][0]["reason"], "profile_member_feature_mismatch")

    def test_profile_label_raw_expression_and_member_failures_cannot_be_waived(self):
        profiles, variants, capture = self.profile_inputs()
        node = capture["data"]["molecularProfiles"]["nodes"][0]
        for field, bad, expected in (("name", "different", "profile_display_name_mismatch"),
                                     ("rawName", "", "profile_raw_name_missing"),
                                     ("rawName", "#VID12 AND #VID99", None),
                                     ("rawName", "BRAF V600E AND KRAS G12D", None)):
            with self.subTest(field=field, value=bad):
                original = node[field]
                node[field] = bad
                result, report = self.run_sweep("civic_profiles", profiles, variant_content=variants, profile_capture=capture, full=True)
                self.assertEqual(result.returncode, 1, result.stderr)
                refusal = report["full_report"]["refusals"][0]
                self.assertEqual(refusal["record"], "civic:9")
                if expected:
                    self.assertEqual(refusal["reason"], expected)
                explanation = {key: refusal[key] for key in ("record", "type", "value", "reason")}
                explanation["source_rule"] = "Synthetic attempted waiver"
                _, waived = self.run_sweep("civic_profiles", profiles, variant_content=variants,
                                           profile_capture=capture, explanations=[explanation])
                self.assertEqual(waived["types"]["profile"]["unexplained"], 1)
                node[field] = original

    def test_full_profile_report_retains_every_missing_companion_id(self):
        text = "molecular_profile_id\tname\tvariant_ids\n"
        text += "".join(f"{identity}\tSource display {identity}\t12\n" for identity in range(1, 15))
        result, report = self.run_sweep("civic_profiles", text, full=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals_omitted"], 4)
        self.assertEqual({row["profile_id"] for row in report["full_report"]["profile_refusals"]},
                         {f"civic:{identity}" for identity in range(1, 15)})

    def test_profile_capture_and_monthly_metadata_cannot_silently_drift(self):
        profiles, variants, capture = self.profile_inputs()
        for old, new, expected in (("\t5\tBRAF", "\t5\tOTHER", "profile_member_feature_mismatch"),
                                   ("\tV600E\t", "\tV600K\t", "profile_member_name_mismatch")):
            result, report = self.run_sweep("civic_profiles", profiles, profile_capture=capture,
                                            variant_content=variants.replace(old, new))
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertEqual(report["profile_refusals"][0]["reason"], expected)
        result, report = self.run_sweep("civic_profiles", profiles, profile_capture=capture,
                                        variant_content=variants.splitlines()[0] + "\n")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"][0]["reason"], "profile_monthly_member_missing")
        result, report = self.run_sweep("civic_profiles", profiles, profile_capture=capture,
                                        variant_content=variants, capture_overrides={"retrieved": "unknown"})
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"][0]["reason"], "profile_companion_provenance_invalid")
        result, report = self.run_sweep("civic_profiles", profiles, profile_capture={"data": None, "errors": [{"message": "Synthetic error"}]},
                                        variant_content=variants)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["profile_refusals"][0]["reason"], "profile_companion_response_invalid")
        result, report = self.run_sweep("civic_profiles", profiles, profile_capture=capture,
                                        variant_content=variants, capture_overrides={"sha256": "0" * 64})
        self.assertEqual(result.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("checksum_mismatch", result.stderr)

    def test_real_native_profile_6153_uses_authoritative_expression_without_monthly_claim(self):
        folder = ROOT / "tests/fixtures/public-dump-sweep"
        pin = json.loads((folder / "manifest.json").read_text())["native_capture"]
        body = (folder / pin["path"]).read_bytes()
        self.assertEqual(hashlib.sha256(body).hexdigest(), pin["sha256"])
        node = next(node for node in json.loads(body)["data"]["molecularProfiles"]["nodes"] if node["id"] == 6153)
        members = [{"level": "source", "record": "civic:" + str(member["id"]),
                    "feature": member["feature"]["name"], "feature_record": "civic:" + str(member["feature"]["id"])}
                   for member in node["variants"]]
        value = {"record": "civic:6153", "label": node["name"], "raw_name": node["rawName"], "members": members}
        result = subprocess.run([BRIDGE], input=json.dumps([{"kind": "profile", "value": value}]) + "\n",
                                text=True, capture_output=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        checked = json.loads(result.stdout)[0]
        self.assertNotIn("reason", checked)
        self.assertEqual(checked["parsed"]["label"], node["name"])
        self.assertEqual(checked["parsed"]["expression"]["source"], node["rawName"])
        self.assertEqual(checked["parsed"]["members"], members)

    def test_observation_revision_and_source_hash_must_match_manifest(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n"
        reply = {"items": [{"answer": {"gene": {"symbol": "BRAF"}},
                             "sections": {"civic": {"data": {"variant_count": 1}}}}]}
        observation = self.observation("civic", reply)
        observation["revision"] = "another-product-revision"
        result, report = self.run_sweep("civic_variants", text, observed=[observation])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"]["product_revision_mismatch"], 1)
        observation["revision"] = "synthetic-product-reply"
        observation["source_sha256"] = ["0" * 64]
        result, report = self.run_sweep("civic_variants", text, observed=[observation])
        self.assertEqual(report["comparisons"]["failures"]["source_hash_mismatch"], 1)

    def test_refusals_do_not_disappear_and_exit_nonzero(self):
        fixture = ROOT / "tests/fixtures/public-dump-sweep/clinvar-edge.tsv"
        result, report = self.run_sweep("clinvar_summary", fixture.read_text())
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["sources"][0]["records"], 2)
        self.assertEqual(report["types"]["clinvar_variation"]["parsed"], 2)
        self.assertEqual(report["types"]["hgvs"]["refused"], 1)
        self.assertEqual(report["refusals"][0]["record"], 2)
        self.assertTrue(report["refusals"][0]["reason"])
        self.assertEqual(report["spelling_mismatches"], 0)

    def test_hash_mismatch_stops_before_parsing(self):
        result, report = self.run_sweep("clinvar_summary", "irrelevant", digest="0" * 64)
        self.assertEqual(result.returncode, 2)
        self.assertIsNone(report)
        self.assertIn("checksum_mismatch", result.stderr)

    def test_bundle_streams_records_and_reports_unknown_sections(self):
        bundle = {"variation": {"clinvar:12": {"id": "clinvar:12", "name": "BRAF"}},
                  "allele": {"ga4gh:VA." + "a" * 32: {
                      "expressions": [{"syntax": "hgvs.c", "value": "NM_004333.6:c.1799T>A"}]}},
                  "futureSection": {"unknown:1": {"value": "do not silently skip"}}}
        result, report = self.run_sweep("clinvar_gks", json.dumps(bundle))
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["sources"][0]["records"], 3)
        self.assertEqual(report["types"]["variant"]["parsed"], 1)
        self.assertEqual(report["types"]["clinvar_variation"]["parsed"], 1)
        self.assertEqual(report["types"]["hgvs"]["parsed"], 1)
        self.assertEqual(report["unmapped_sections"], {"futureSection": 1})

    def test_sample_limit_marks_run_incomplete(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n2\tG12D\tKRAS\t\n"
        result, report = self.run_sweep("civic_variants", text, limit=1)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertFalse(report["complete"])
        self.assertEqual(report["gene_counts"], {"BRAF": 1})

    def test_clingen_evidence_keeps_disease_and_only_explicit_criterion_citations(self):
        record = {"uuid": "synthetic", "condition": {"id": "MONDO:0004992", "label": "cancer"},
                  "outside": "PMID: 999", "evidenceLine": [
                      {"type": "CriterionAssessment", "text": "PMIDs: 123, 456"}]}
        result, report = self.run_sweep("clingen_evidence", json.dumps(record))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["types"]["disease"]["parsed"], 1)
        self.assertEqual(report["types"]["citation"]["parsed"], 2)

    def test_civic_evidence_checks_disease_therapy_citation_and_free_text(self):
        text = ("evidence_id\tmolecular_profile_id\tsource_type\tcitation_id\tdisease\tdoid\ttherapies\tevidence_statement\n"
                "1\t9\tPubMed\t123\tcancer\t162\tdrug one,drug two\t  original source text  \n")
        result, report = self.run_sweep("civic_evidence", text)
        self.assertEqual(result.returncode, 0, result.stderr)
        for kind, count in (("disease", 1), ("drug", 2), ("citation", 1), ("source_text", 1)):
            self.assertEqual(report["types"][kind]["parsed"], count, kind)

    def test_clingen_summary_checks_reference_qualified_notations_and_ids(self):
        text = ("HGNC Gene Symbol\tDisease\tMondo Id\tHGVS Expressions\tAllele Registry Id\tClinVar Variation Id\n"
                "BRAF\tcancer\tMONDO:0004992\tNM_004333.6:c.1799T>A\tCA123456\t13961\n")
        result, report = self.run_sweep("clingen_summary", text)
        self.assertEqual(result.returncode, 0, result.stderr)
        for kind in ("gene", "disease", "hgvs", "ca", "clinvar_variation"):
            self.assertEqual(report["types"][kind]["parsed"], 1, kind)

    def observation(self, source, result, record=None, release="synthetic-edge-sample"):
        return {"source": source, "release": release, "revision": "synthetic-product-reply",
                "retrieved": "2026-10-29", "record": record, "reply": {"result": result}}

    def connection(self, nodes):
        return {"pages": [{"after": None, "totalCount": len(nodes), "nodes": nodes,
                           "pageInfo": {"hasNextPage": False, "endCursor": None}}]}

    def gene_variants(self, gene, variants):
        nodes = []
        for identity, evidence, statuses in variants:
            assertions = [{"id": index + 1, "status": status} for index, status in enumerate(statuses)]
            profile = {"id": identity, "evidenceCountsByStatus": {"acceptedCount": evidence},
                       "assertions": self.connection(assertions)}
            nodes.append({"id": identity, "__typename": "GeneVariant", "deprecated": False,
                          "feature": {"id": 9, "name": gene}, "molecularProfiles": self.connection([profile])})
        return {"symbol": gene, "feature": {"id": 9, "name": gene}, "variants": self.connection(nodes)}

    def civic_observation(self, gene, total, variants):
        result = {"items": [{"answer": {"gene": {"symbol": gene}},
                             "sections": {"civic": {"data": {"id": 9, "variant_count": total}}}}]}
        observation = self.observation("civic", result)
        observation["gene_variants"] = [self.gene_variants(gene, variants)]
        return observation

    def test_compares_saved_gene_reply_and_reports_missing_and_wrong_counts(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n2\tG12D\tKRAS\t\n"
        observed = self.civic_observation("BRAF", 9, [(1, 1, [])])
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"],
                         {"civic_raw_total_mismatch": 1, "civic_gene_unobserved": 1})

    def test_accepted_scope_uses_evidence_or_assertions_and_retains_raw_differences(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n2\tV600K\tBRAF\t\n"
        observed = self.civic_observation("BRAF", 4, [(1, 1, []), (2, 0, ["ACCEPTED"]),
                                                        (3, 0, ["SUBMITTED"]), (4, 0, [])])
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 0, result.stderr)
        comparison = report["comparisons"]["civic_gene_comparisons"][0]
        self.assertEqual((comparison["tsv_raw_total"], comparison["api_raw_total"], comparison["api_accepted_total"]), (2, 4, 2))
        self.assertEqual(comparison["api_only_raw_ids"], ["civic:3", "civic:4"])
        self.assertEqual(comparison["excluded_api_ids"], ["civic:3", "civic:4"])
        self.assertEqual(comparison["api_only_accepted_ids"], [])

    def test_equal_accepted_counts_do_not_hide_unmatched_variant_ids(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n"
        observed = self.civic_observation("BRAF", 1, [(2, 1, [])])
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 1, result.stderr)
        comparison = report["comparisons"]["civic_gene_comparisons"][0]
        self.assertEqual(comparison["tsv_only_accepted_ids"], ["civic:1"])
        self.assertEqual(comparison["api_only_accepted_ids"], ["civic:2"])
        self.assertEqual(report["comparisons"]["failures"], {"civic_accepted_variant_ids_mismatch": 1})

    def test_count_only_or_incomplete_gene_variants_capture_cannot_prove_scope(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n"
        observed = self.civic_observation("BRAF", 1, [(1, 1, [])])
        observed.pop("gene_variants")
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"], {"civic_gene_variants_missing": 1})
        observed = self.civic_observation("BRAF", 2, [(1, 1, [])])
        observed["gene_variants"][0]["variants"]["pages"][0]["pageInfo"] = {"hasNextPage": True, "endCursor": "more"}
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"], {"civic_connection_incomplete": 1})

    def test_full_report_retains_every_unmatched_id_with_bounded_stdout(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n"
        text += "".join(f"{identity}\tV\tBRAF\t\n" for identity in range(1, 31))
        observed = self.civic_observation("BRAF", 30, [(identity, 1, []) for identity in range(101, 131)])
        result, report = self.run_sweep("civic_variants", text, observed=[observed], full=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        row = report["comparisons"]["civic_gene_comparisons"][0]
        self.assertEqual(row["tsv_only_accepted_ids_omitted"], 10)
        full = report["full_report"]["comparisons"]["civic_gene_comparisons"][0]
        self.assertEqual(set(full["tsv_only_accepted_ids"]), {f"civic:{identity}" for identity in range(1, 31)})
        self.assertEqual(set(full["api_only_accepted_ids"]), {f"civic:{identity}" for identity in range(101, 131)})

    def test_direct_gene_scope_excludes_fusion_partner_participation(self):
        text = ("variant_id\tvariant\tgene\thgvs_descriptions\t5_prime_partner\t3_prime_partner\n"
                "1\tFusion\t\t\tBCR\tABL1\n")
        observed = self.civic_observation("BCR", 0, [])
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["gene_counts"], {})
        self.assertEqual(report["types"]["gene"]["parsed"], 2)
        observed = self.civic_observation("BCR", 1, [(1, 1, [])])
        observed["gene_variants"][0]["variants"]["pages"][0]["nodes"][0]["feature"]["name"] = "BCR::ABL1"
        result, report = self.run_sweep("civic_variants", text, observed=[observed])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"], {"civic_variant_direct_gene_mismatch": 1})

    def test_compares_clinvar_ids_through_source_graph_and_rejects_changed_spelling(self):
        allele = "ga4gh:VA." + "a" * 32
        bundle = {"variation": {"clinvar:12": {"members": ["#/allele/" + allele]}},
                  "allele": {allele: {"name": "source allele"}}}
        reply = {"candidates": [{"identity": {"ids": ["clinvar:12", allele]},
                                  "classification": "CAUTION is prose, not an allele ID"}]}
        observation = self.observation("clinvar", reply, "clinvar:12")
        result, report = self.run_sweep("clinvar_gks", json.dumps(bundle), observed=[observation])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["comparisons"]["checked"]["clinvar_ids"], 2)
        self.assertEqual(report["comparisons"]["product_revision"], "synthetic-product-reply")
        self.assertEqual(report["comparisons"]["provenance"][0]["observations"], 1)
        reply["candidates"][0]["identity"]["ids"][0] = "clinvar:012"
        result, report = self.run_sweep("clinvar_gks", json.dumps(bundle), observed=[observation])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"], {"clinvar_identifier_spelling_mismatch": 1})

    def test_source_date_drift_never_counts_as_equality(self):
        text = "variant_id\tvariant\tgene\thgvs_descriptions\n1\tV600E\tBRAF\t\n"
        observed = [self.observation("civic", {"items": []}, release="different-source-date")]
        result, report = self.run_sweep("civic_variants", text, observed=observed)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["comparisons"]["failures"]["source_release_mismatch"], 1)

    def test_only_exact_publisher_explanations_allow_refused_values(self):
        text = (ROOT / "tests/fixtures/public-dump-sweep/clinvar-edge.tsv").read_text()
        _, initial = self.run_sweep("clinvar_summary", text)
        refusal = initial["refusals"][0]
        explanation = {key: refusal[key] for key in ("record", "type", "value", "reason")}
        explanation["source_rule"] = "HGVS descriptions require a body: https://hgvs-nomenclature.org/stable/"
        result, report = self.run_sweep("clinvar_summary", text, explanations=[explanation])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["types"]["hgvs"]["explained"], 1)
        explanation["value"] = "another input"
        result, report = self.run_sweep("clinvar_summary", text, explanations=[explanation])
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report["types"]["hgvs"]["unexplained"], 1)

    def test_gkm_composite_proposition_keys_and_typed_codings(self):
        # Synthetic records follow the two documented GKS/GKM section inventories.
        bundle = {"varcond-proposition": {"VCV000000001.1-G-PATH": {
            "id": "VCV000000001.1-G-PATH", "subjectVariant": "#/variation/clinvar:12"}},
            "variation": {"clinvar:12": {"mappings": [{"coding": {
                "system": "https://www.ncbi.nlm.nih.gov/snp/", "code": "1799945"}}]}},
            "sequenceReference": {"SQ.example": {"type": "SequenceReference"}}}
        result, report = self.run_sweep("clinvar_gks", json.dumps(bundle))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["types"]["native_identifier"]["parsed"], 2)
        self.assertEqual(report["types"]["rs"]["parsed"], 1)

    def test_official_cc0_example_checks_all_published_hgvs_expressions(self):
        folder = ROOT / "tests/fixtures/public-dump-sweep"
        source = json.loads((folder / "manifest.json").read_text())["sources"][0]
        data = (folder / source["path"]).read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(), source["sha256"])
        result, report = self.run_sweep("clinvar_gks_record", data.decode())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["types"]["hgvs"]["parsed"], 18)
        self.assertEqual(report["types"]["expression"]["parsed"], 4)
        self.assertEqual(report["types"]["gene"]["parsed"], 1)

    def test_official_monthly_civic_headers_and_rows_parse_without_repair(self):
        folder = ROOT / "tests/fixtures/public-dump-sweep"
        sources = json.loads((folder / "manifest.json").read_text())["sources"]
        for source in sources:
            if not source["kind"].startswith("civic_"):
                continue
            with self.subTest(kind=source["kind"]):
                data = (folder / source["path"]).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(), source["sha256"])
                result, report = self.run_sweep(source["kind"], data.decode())
                self.assertEqual(result.returncode, int(source["kind"] == "civic_profiles"), result.stderr + str(report))
                self.assertEqual(report["sources"][0]["records"], 1)


if __name__ == "__main__":
    unittest.main()
