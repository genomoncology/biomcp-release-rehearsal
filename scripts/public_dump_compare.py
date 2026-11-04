"""Compare saved CLI replies to source identities using a temporary disk index."""
from collections import Counter
from datetime import date
import hashlib
import json
import sqlite3
from public_dump_civic_scope import CivicScope, ScopeError

VARIANT_PREFIXES = ("clinvar:", "VCV", "CA", "PA", "rs", "COSV", "COSM", "COSN", "ga4gh:VA.")


def strings(value):
    if isinstance(value, dict):
        for child in value.values():
            yield from strings(child)
    elif isinstance(value, list):
        for child in value:
            yield from strings(child)
    elif isinstance(value, str):
        yield value


def printed_variant_ids(value):
    """Read typed variant identity slots, excluding prose and attribution records."""
    if isinstance(value, dict):
        for key, child in value.items():
            if key == "ids" and isinstance(child, list):
                yield from (identity for identity in child if isinstance(identity, str))
            elif key == "requested" and isinstance(child, str):
                yield child
            elif key == "id" and value.get("level") in {"allele", "protein"}:
                yield child
            else:
                yield from printed_variant_ids(child)
    elif isinstance(value, list):
        for child in value:
            yield from printed_variant_ids(child)


class Comparison:
    """Index source graph identities outside the repository and outside memory."""

    def __init__(self):
        self.db = sqlite3.connect("")
        self.db.executescript("""
            CREATE TABLE ids(node TEXT, id TEXT, PRIMARY KEY(node,id));
            CREATE TABLE edges(node TEXT, target TEXT, PRIMARY KEY(node,target));
        """)
        self.releases = {}
        self.counts = Counter()
        self.examples = []
        self.checked = Counter()
        self.civic_scope = CivicScope(self.db)
        self.civic_rows = []

    def add(self, context, kind, release):
        source = "civic" if kind.startswith("civic_") else "clinvar" if kind.startswith("clinvar_") else "clingen"
        self.releases.setdefault(source, set()).add(release)
        if kind == "civic_variants" and context:
            self.civic_scope.add(context)
            return
        if context:
            node, values, edges = context
            self.db.executemany("INSERT OR IGNORE INTO ids VALUES (?,?)", ((node, value) for value in values))
            self.db.executemany("INSERT OR IGNORE INTO edges VALUES (?,?)", ((node, target) for target in edges))

    def refuse(self, reason, identity):
        self.counts[reason] += 1
        if len(self.examples) < 10:
            self.examples.append({"reason": reason, "record": str(identity)[:512]})

    def allowed(self, record):
        root = "variation/" + record
        rows = self.db.execute("""
            WITH RECURSIVE reachable(node) AS (
                SELECT ? UNION SELECT edges.target FROM edges JOIN reachable ON edges.node=reachable.node
            ) SELECT DISTINCT id FROM ids JOIN reachable USING(node)
        """, (root,))
        return {row[0] for row in rows}

    def compare(self, path, genes, revision, sources):
        if not isinstance(revision, str) or not revision:
            raise ValueError("manifest_missing_product_revision")
        hashes = {}
        for source, _ in sources:
            family = "civic" if source["kind"].startswith("civic_") else "clinvar" if source["kind"].startswith("clinvar_") else "clingen"
            hashes.setdefault(family, set()).add(source["sha256"])
        provenance = Counter()
        seen_genes = set()
        digest = hashlib.sha256()
        with path.open("rb") as raw:
            for chunk in iter(lambda: raw.read(1024 * 1024), b""):
                digest.update(chunk)
        with path.open(encoding="utf-8") as handle:
            for number, line in enumerate(handle, 1):
                observation = json.loads(line)
                source = observation["source"]
                if len(self.releases.get(source, set())) > 1:
                    self.refuse("source_inputs_have_different_releases", source)
                    continue
                if observation["release"] not in self.releases.get(source, set()):
                    self.refuse("source_release_mismatch", number)
                    continue
                if not observation.get("revision") or not observation.get("retrieved"):
                    raise ValueError("observation_missing_revision_or_retrieval_date")
                if observation["revision"] != revision:
                    self.refuse("product_revision_mismatch", number)
                    continue
                if observation.get("source_sha256") != sorted(hashes.get(source, set())):
                    self.refuse("source_hash_mismatch", number)
                    continue
                date.fromisoformat(observation["retrieved"])
                provenance[(source, observation["release"], observation["retrieved"])] += 1
                reply = observation["reply"]
                if not isinstance(reply, dict) or "result" not in reply:
                    raise ValueError("observation_not_cli_envelope")
                if source == "civic":
                    self.civic(reply["result"], genes, seen_genes, observation.get("gene_variants"))
                elif source == "clinvar":
                    self.clinvar(reply["result"], observation["record"])
                else:
                    raise ValueError("observation_unsupported_source")
        for gene in genes.keys() - seen_genes:
            self.refuse("civic_gene_unobserved", gene)
            self.civic_rows.append(self.civic_scope.missing(gene, genes[gene]))
        if "clinvar" in self.releases and not self.checked["clinvar_ids"]:
            self.refuse("clinvar_ids_unobserved", "clinvar")
        return {"observed_sha256": digest.hexdigest(), "product_revision": revision,
                "provenance": [{"source": source, "release": release, "retrieved": retrieved,
                                "source_sha256": sorted(hashes[source]), "observations": count}
                               for (source, release, retrieved), count in provenance.items()],
                "checked": dict(self.checked),
                "civic_gene_comparisons": self.civic_rows,
                "failures": dict(self.counts), "examples": self.examples}

    def civic(self, result, genes, seen, captures):
        if captures is not None and (not isinstance(captures, list) or any(not isinstance(row, dict) for row in captures)):
            raise ValueError("civic_gene_variants_invalid")
        captured = {}
        for capture in captures or []:
            symbol = capture.get("symbol")
            if symbol in captured:
                self.refuse("civic_duplicate_gene_variants_capture", symbol)
            captured[symbol] = capture
        for row in result.get("items", []):
            gene = row.get("answer", {}).get("gene", {}).get("symbol")
            section = row.get("sections", {}).get("civic", {})
            actual = section.get("data", {}).get("variant_count")
            if not gene or actual is None:
                self.refuse("civic_gene_or_count_missing", gene)
                continue
            seen.add(gene)
            capture = captured.pop(gene, None)
            try:
                if capture is None:
                    raise ScopeError("civic_gene_variants_missing")
                report, failures = self.civic_scope.compare(capture, gene, section["data"]["id"], actual, genes.get(gene, 0))
                self.civic_rows.append(report)
                self.checked["civic_genes"] += 1
                for reason in failures:
                    self.refuse(reason, gene)
            except (KeyError, TypeError, ValueError) as error:
                reason = str(error) if isinstance(error, ScopeError) else "civic_gene_variants_invalid"
                self.refuse(reason, gene)
                self.civic_rows.append(self.civic_scope.missing(gene, genes.get(gene, 0)))
        for gene in captured:
            self.refuse("civic_unbound_gene_variants_capture", gene)

    def clinvar(self, result, record):
        allowed = self.allowed(record)
        if not allowed:
            self.refuse("clinvar_record_not_in_dump", record)
            return
        printed = list(printed_variant_ids(result))
        if not printed:
            self.refuse("clinvar_reply_has_no_variant_ids", record)
        for value in printed:
            self.checked["clinvar_ids"] += 1
            if value not in allowed:
                self.refuse("clinvar_identifier_spelling_mismatch", f"{record}: {value}")

    def close(self):
        self.db.close()
