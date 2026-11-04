"""Bind native CIViC rawName captures to exact monthly TSV inputs.

The capture retrieval is independent of the monthly release. This module checks
source identities and feature/member equality, never Boolean display grammar.
Variant metadata and captured nodes use a temporary disk index. --max-records
also bounds the variant metadata scan, so missing members remain refusals.
"""
import csv
from datetime import datetime
import gzip
import hashlib
import json
import sqlite3


def pinned_file(manifest, entry):
    path = (manifest.parent / entry["path"]).resolve()
    if any((parent / ".git").exists() for parent in path.parents):
        raise ValueError("dump_must_be_outside_git")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    if digest.hexdigest() != entry["sha256"]:
        raise ValueError(f"checksum_mismatch: {path}")
    return path


def native_id(value):
    return type(value) is int and value > 0


def tsv_id(value):
    return isinstance(value, str) and value.isascii() and value.isdigit() and not value.startswith("0")


class ProfileCompanions:
    def __init__(self, manifest, sources, maximum):
        self.db = sqlite3.connect("")
        self.db.execute("CREATE TABLE variants (release TEXT, id TEXT, value TEXT)")
        self.db.execute("CREATE INDEX variant_id ON variants(release,id)")
        self.db.execute("CREATE TABLE profiles (id TEXT, value TEXT, reason TEXT)")
        self.db.execute("CREATE INDEX profile_id ON profiles(id)")
        self.db.execute("CREATE TABLE monthly_profiles (id TEXT PRIMARY KEY)")
        self.provenance = []
        self.invalid = None
        try:
            expected = sorted(source["sha256"] for source, _ in sources
                              if source["kind"] in {"civic_profiles", "civic_variants"})
            for entry in json.loads(manifest.read_text()).get("profile_companions", []):
                self.load(manifest, entry, expected)
            if self.provenance:
                self.variants(sources, maximum)
            self.db.commit()
        except BaseException:
            self.close()
            raise

    def load(self, manifest, entry, expected):
        if not isinstance(entry, dict) or not {"path", "sha256", "url", "retrieved", "source_sha256"} <= entry.keys():
            self.invalid = "profile_companion_provenance_missing"
            return
        path = pinned_file(manifest, entry)
        reason = None
        try:
            datetime.fromisoformat(entry["retrieved"].replace("Z", "+00:00"))
            if not entry["url"].startswith("https://"):
                raise ValueError("capture_url")
        except (ValueError, AttributeError, TypeError):
            reason = "profile_companion_provenance_invalid"
        if entry["source_sha256"] != expected:
            reason = "profile_companion_source_hash_mismatch"
        self.provenance.append(dict(entry))
        body = json.loads(path.read_text())
        data = body.get("data") if isinstance(body, dict) else None
        profiles = data.get("molecularProfiles") if isinstance(data, dict) else None
        nodes = profiles.get("nodes") if isinstance(profiles, dict) else None
        if not isinstance(nodes, list) or body.get("errors"):
            self.invalid = "profile_companion_response_invalid"
            return
        for node in nodes:
            if not isinstance(node, dict) or not native_id(node.get("id")):
                self.invalid = "profile_companion_identity_invalid"
                continue
            self.db.execute("INSERT INTO profiles VALUES (?,?,?)", (str(node["id"]), json.dumps(node), reason))

    def variants(self, sources, maximum):
        for source, path in sources:
            if source["kind"] != "civic_variants":
                continue
            opener = gzip.open if path.suffix == ".gz" else open
            with opener(path, "rt", encoding="utf-8", newline="") as handle:
                for index, row in enumerate(csv.DictReader(handle, delimiter="\t")):
                    if maximum is not None and index >= maximum:
                        break
                    if None in row or any(value is None for value in row.values()):
                        raise ValueError("malformed_tsv_record")
                    self.db.execute("INSERT INTO variants VALUES (?,?,?)",
                                    (source["release"], row["variant_id"], json.dumps(row)))

    def resolve(self, original, release):
        value = original["value"]
        identity = value["record"].removeprefix("civic:")
        self.db.execute("INSERT OR IGNORE INTO monthly_profiles VALUES (?)", (identity,))
        rows = self.db.execute("SELECT value,reason FROM profiles WHERE id=?", (identity,)).fetchall()
        reason = self.invalid
        if not tsv_id(identity):
            reason = "profile_identity_invalid"
        elif not reason and len(rows) != 1:
            reason = "profile_companion_missing" if not rows else "profile_companion_ambiguous"
        if not reason:
            raw, reason = rows[0]
            node = json.loads(raw)
            if not reason:
                value, reason = self.match(value, node, release)
        result = {"kind": "profile", "value": value}
        if reason:
            result["mapping_reason"] = reason
        return result

    def unmatched(self):
        """List companion identities absent from the complete monthly input."""
        return self.db.execute("SELECT DISTINCT id FROM profiles WHERE id NOT IN (SELECT id FROM monthly_profiles) ORDER BY id")

    def match(self, value, node, release):
        members = node.get("variants")
        ids = value["members"]
        if not isinstance(members, list) or not members or any(not isinstance(member, dict) or not native_id(member.get("id")) for member in members):
            return value, "profile_member_ids_invalid"
        native_ids = [str(member["id"]) for member in members]
        if not ids or any(not tsv_id(identity) for identity in ids) or len(set(ids)) != len(ids) or len(set(native_ids)) != len(native_ids) or set(ids) != set(native_ids):
            return value, "profile_member_ids_mismatch"
        if node.get("name") != value["label"]:
            return value, "profile_display_name_mismatch"
        if not isinstance(node.get("rawName"), str) or not node["rawName"].strip():
            return value, "profile_raw_name_missing"
        mapped = []
        for member in members:
            result, reason = self.member(member, release)
            if reason:
                return value, reason
            mapped.append(result)
        return {**value, "members": mapped, "raw_name": node["rawName"]}, None

    def member(self, member, release):
        rows = self.db.execute("SELECT value FROM variants WHERE release=? AND id=?", (release, str(member["id"]))).fetchall()
        if len(rows) != 1:
            return None, "profile_monthly_member_missing" if not rows else "profile_monthly_member_ambiguous"
        row = json.loads(rows[0][0])
        feature = member.get("feature")
        if not isinstance(feature, dict) or not native_id(feature.get("id")) or not tsv_id(row.get("feature_id")) or not feature.get("name") or str(feature["id"]) != row["feature_id"] or feature["name"] != row.get("feature_name"):
            return None, "profile_member_feature_mismatch"
        if not member.get("name") or member["name"] != row.get("variant"):
            return None, "profile_member_name_mismatch"
        return {"level": "source", "record": "civic:" + str(member["id"]),
                "feature": feature["name"], "feature_record": "civic:" + str(feature["id"])}, None

    def close(self):
        self.db.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
