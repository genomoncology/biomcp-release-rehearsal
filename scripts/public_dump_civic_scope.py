"""Compare direct CIViC variant identities in the publisher's accepted scope.

The saved gene_variants companion retains symbol, feature {id,name}, and a
variants connection. Every connection has pages containing the request after,
native totalCount, pageInfo {hasNextPage,endCursor}, and nodes. Variant nodes
retain id,__typename,deprecated,feature, and a molecularProfiles connection.
Profiles retain id,evidenceCountsByStatus.acceptedCount, and an assertions
connection whose nodes retain id,status. No count-only capture proves membership.

Publisher definitions: server/app/graphql/{resolvers/gene_variants.rb,
types/interfaces/variant_interface.rb,types/entities/molecular_profile_type.rb}
in https://github.com/griffithlab/civic-v2 . GeneVariants excludes deprecated
variants. Profiles' assertions can include SUBMITTED as well as ACCEPTED records.
"""


class ScopeError(ValueError):
    """A named missing or inconsistent source observation."""


def number(value, *, positive=False):
    if type(value) is not int or value < int(positive):
        raise ScopeError("civic_capture_number_invalid")
    return value


def connection(capture):
    """Validate every page and native identity before using a source connection."""
    if not isinstance(capture, dict):
        raise ScopeError("civic_connection_invalid")
    pages = capture.get("pages")
    if not isinstance(pages, list) or not pages:
        raise ScopeError("civic_connection_incomplete")
    nodes, identities, cursors = [], set(), set()
    after = total = None
    for index, page in enumerate(pages):
        if "after" not in page or page["after"] != after:
            raise ScopeError("civic_connection_cursor_mismatch")
        current = number(page["totalCount"])
        if total is not None and current != total:
            raise ScopeError("civic_connection_total_changed")
        total = current
        info = page["pageInfo"]
        more = info["hasNextPage"]
        if type(more) is not bool or not isinstance(page["nodes"], list):
            raise ScopeError("civic_connection_invalid")
        if more and index == len(pages) - 1:
            raise ScopeError("civic_connection_incomplete")
        if not more and index != len(pages) - 1:
            raise ScopeError("civic_connection_after_terminal_page")
        for node in page["nodes"]:
            identity = number(node["id"], positive=True)
            if identity in identities:
                raise ScopeError("civic_connection_duplicate_id")
            identities.add(identity)
            nodes.append(node)
        if more:
            cursor = info["endCursor"]
            if not isinstance(cursor, str) or not cursor or cursor in cursors:
                raise ScopeError("civic_connection_cursor_mismatch")
            cursors.add(cursor)
            after = cursor
    if len(nodes) != total:
        raise ScopeError("civic_connection_total_mismatch")
    return nodes, total


def accepted(node):
    profiles, _ = connection(node["molecularProfiles"])
    result = False
    for profile in profiles:
        evidence = number(profile["evidenceCountsByStatus"]["acceptedCount"])
        assertions, _ = connection(profile["assertions"])
        if any(not isinstance(row["status"], str) or not row["status"] for row in assertions):
            raise ScopeError("civic_assertion_status_missing")
        result |= evidence > 0 or any(row["status"] == "ACCEPTED" for row in assertions)
    return result


def source_ids(identities):
    return ["civic:" + identity for identity in sorted(identities)]


class CivicScope:
    """Keep TSV membership on disk and compare one direct gene at a time."""

    def __init__(self, db):
        self.db = db
        db.execute("CREATE TABLE civic_variants(gene TEXT, id TEXT, PRIMARY KEY(gene,id))")

    def add(self, context):
        self.db.executemany("INSERT OR IGNORE INTO civic_variants VALUES (?,?)",
                            ((gene, context["id"]) for gene in context["genes"]))

    def ids(self, gene):
        return {row[0] for row in self.db.execute("SELECT id FROM civic_variants WHERE gene=?", (gene,))}

    def missing(self, gene, raw):
        return {"gene": gene, "tsv_raw_total": raw, "api_raw_total": None,
                "api_accepted_total": None, "uncompared_tsv_ids": source_ids(self.ids(gene))}

    def compare(self, capture, gene, feature_id, raw_total, tsv_total):
        feature = capture["feature"]
        if feature["name"] != gene or number(feature["id"], positive=True) != feature_id:
            raise ScopeError("civic_gene_feature_mismatch")
        nodes, api_total = connection(capture["variants"])
        api_ids, accepted_ids = set(), set()
        for node in nodes:
            if node["__typename"] != "GeneVariant" or node["deprecated"] is not False:
                raise ScopeError("civic_variant_scope_invalid")
            if node["feature"]["id"] != feature["id"] or node["feature"]["name"] != gene:
                raise ScopeError("civic_variant_direct_gene_mismatch")
            identity = str(node["id"])
            api_ids.add(identity)
            if accepted(node):
                accepted_ids.add(identity)
        tsv_ids = self.ids(gene)
        report = {"gene": gene, "tsv_raw_total": tsv_total, "api_raw_total": api_total,
                  "product_raw_total": raw_total, "api_accepted_total": len(accepted_ids),
                  "tsv_only_raw_ids": source_ids(tsv_ids - api_ids),
                  "api_only_raw_ids": source_ids(api_ids - tsv_ids),
                  "excluded_api_ids": source_ids(api_ids - accepted_ids),
                  "tsv_only_accepted_ids": source_ids(tsv_ids - accepted_ids),
                  "api_only_accepted_ids": source_ids(accepted_ids - tsv_ids)}
        failures = []
        if number(raw_total) != api_total:
            failures.append("civic_raw_total_mismatch")
        if len(tsv_ids) != tsv_total:
            failures.append("civic_duplicate_tsv_variant_id")
        if tsv_ids != accepted_ids:
            failures.append("civic_accepted_variant_ids_mismatch")
        return report, failures
