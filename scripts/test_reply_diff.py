"""Exercise reply comparison through its command line."""
import hashlib
import json
from pathlib import Path
import unittest

from reply_diff_acquisition_tests import ReplyAcquisitions
from reply_diff_test_support import ReplyDiffCase


class ReplyDiff(ReplyDiffCase):
    def test_reply_and_exit_changes_name_the_case_pointer(self):
        key = "case/schema/help.yaml/step/0"
        self.write(self.base, key)
        for stdout, code, pointer in [
            (b'{"status":"completE"}\n', 0, "/0/stdout/status"),
            (b'{"status":"complete"}\n', 7, "/0/exit"),
            (b'{ "status":"complete"}\n', 0, "/0/stdout/@character:"),
        ]:
            with self.subTest(pointer=pointer):
                self.write(self.head, key, stdout, code)
                result = self.compare()
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertIn(key, result.stdout)
                self.assertIn(pointer, result.stdout)

    def test_trial_output_and_arguments_remain_significant(self):
        key = "trial/housekeeping/version/" + "a" * 64 + "/0"
        self.write(self.base, key, b"original\n")
        for output, args in [(b"changed\n", ["help"]), (b"original\n", ["version"])]:
            self.write(self.head, key, output, args=args)
            result = self.compare()
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn(key, result.stdout)

    def test_build_identity_and_owned_paths_are_masked(self):
        key = "case/version.yaml/step/0"
        for directory, suffix in [(self.base, "base"), (self.head, "head")]:
            root = "/tmp/" + suffix
            output = f"biomcp 1.{suffix}\nCommit: {suffix}\nConfig: {root}/config\n"
            self.write(directory, key, output.encode(), args=["version", "--verbose"],
                       version="1." + suffix, root=root)
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_data_colliding_with_build_metadata_is_never_masked(self):
        key = "case/source.yaml/step/0"
        for field in ["version", "executable", "root", "checkout"]:
            with self.subTest(field=field):
                for directory, value in [(self.base, "1.0.0"), (self.head, "1.0.1")]:
                    self.write(directory, key, json.dumps({"source": value}).encode(),
                               **{field: value})
                result = self.compare()
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertIn("/0/stdout/source", result.stdout)

    def runtime_capture(self, directory, suffix, *, bad_reference=False,
                        lifetime=90, status="complete"):
        root = "/tmp/" + suffix
        pool = suffix + "-pool"
        common = dict(root=root, args=["search", "trial"])
        self.write(directory, "case/runtime.yaml/step/0", json.dumps({
            "page": {"pool": {"id": pool, "created": 10 if suffix == "base" else 20,
                                "expires": (10 if suffix == "base" else 20) + lifetime, "size": 1}},
            "result": {"fetched": 30 if suffix == "base" else 40},
            "next": [{"kind": "page", "args": ["search", "trial", "--pool", pool]}],
            "status": status,
        }).encode(), **common)
        self.write(directory, "case/runtime.yaml/step/1", json.dumps({
            "query": {"pool": pool},
            "result": {"snapshot": {"revision": "same", "acquired": 50 if suffix == "base" else 60}},
        }).encode(), root=root,
            args=["search", "trial", "--pool", "broken" if bad_reference else pool])
        markdown = "# Trial\n**fetched:** " + ("30" if suffix == "base" else "40") + "\n"
        self.write(directory, "case/runtime.yaml/step/2", markdown.encode(), **common)
        self.write(directory, "case/runtime.yaml/step/3", json.dumps({
            "result": {"file": {"data": {"path": root + "/reply.json"}}},
        }).encode(), root=root, args=["get", "trial", "--save", "reply.json"])

    def test_runtime_values_bind_across_the_whole_capture(self):
        self.runtime_capture(self.base, "base")
        self.runtime_capture(self.head, "head")
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertEqual(result.stdout, "")

    def test_runtime_binding_rejects_changed_reference_lifetime_and_data(self):
        self.runtime_capture(self.base, "base")
        for mutation in [dict(bad_reference=True), dict(lifetime=91), dict(status="partial")]:
            with self.subTest(mutation=mutation):
                self.runtime_capture(self.head, "head", **mutation)
                result = self.compare()
                self.assertNotEqual(result.returncode, 0, result.stdout)

    def test_record_path_hash_keys_pair_only_through_declared_names(self):
        def key(args, root=None):
            if root:
                args = [a.replace(root, '<root>') for a in args]
            digest = hashlib.sha256(json.dumps(args, separators=(",", ":")).encode()).hexdigest()
            return "trial/cache/replay/" + digest + "/0"
        for directory, suffix in [(self.base, "base"), (self.head, "head")]:
            root = "/tmp/" + suffix
            args = ["get", "gene", "--record", root + "/record"]
            self.write(directory, key(args, root), args=args, root=root)
            other = "/tmp/other-" + suffix
            args = ["get", "gene", "--config", other + "/config", "--replay", root + "/record"]
            self.write(directory, key(args, other), args=args, root=other)
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        args = ["get", "gene", "--config", "/tmp/other-head/config", "--replay", "/tmp/head/record"]
        changed = args[:-1] + ["/tmp/head/another"]
        self.write(self.head, key(args, '/tmp/other-head'), args=changed, root="/tmp/other-head")
        self.assertEqual(self.compare().returncode, 1)

    def test_json_source_text_and_literal_arguments_remain_significant(self):
        key = 'case/literal.yaml/step/0'
        for place in ['source', 'args', 'mcp-source', 'source-args', 'source-clock',
                      'source-continuation', 'source-snapshot']:
            with self.subTest(place=place):
                for directory, clock in [(self.base, 10), (self.head, 20)]:
                    literal = json.dumps({'fetched': clock})
                    output = {'result': {'description': literal}}
                    extra = {}
                    if place == 'args':
                        output = {'status': 'complete'}
                        extra['args'] = ['search', 'article', literal]
                    elif place == 'mcp-source':
                        output = {'jsonrpc': '2.0', 'id': 1, 'result': {
                            'structuredContent': output,
                            'content': [{'type': 'text', 'text': json.dumps(output)}]}}
                        extra['surface'] = 'mcp'
                    elif place == 'source-args':
                        output = {'result': {'args': ['search', 'trial', '--pool', literal]}}
                    elif place == 'source-clock':
                        output = {'result': {'notes': {'fetched': clock}}}
                    elif place == 'source-continuation':
                        output = {'result': {'items': [{'sections': {'locations': {'data': {
                            'continuation': {'pool': literal}}}}}]}}
                    elif place == 'source-snapshot':
                        output = {'next': [{'kind': 'section', 'args': [
                            'get', 'phenotype', 'HP:0000001', '--snapshot', literal]}]}
                    self.write(directory, key, json.dumps(output).encode(), **extra)
                result = self.compare()
                self.assertEqual(result.returncode, 1, result.stderr + result.stdout)

    def test_mcp_runtime_fields_bind_only_inside_transport_frames(self):
        for directory, suffix in [(self.base, 'base'), (self.head, 'head')]:
            pool = suffix + '-pool'
            reply = {'page': {'pool': {'id': pool, 'created': 10 if suffix == 'base' else 20,
                                      'expires': 100 if suffix == 'base' else 110, 'size': 1}},
                     'query': {'pool': pool}}
            frame = {'jsonrpc': '2.0', 'id': 1, 'result': {
                'structuredContent': reply,
                'content': [{'type': 'text', 'text': json.dumps(reply)}]}}
            request = {'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call', 'params': {
                'name': 'biomcp', 'arguments': {'args': ['search', 'trial', '--pool', pool]}}}
            self.write(directory, 'case/mcp.yaml/step/0', json.dumps(frame).encode(),
                       surface='mcp', args=['serve', '<requests>', json.dumps(request)])
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_mcp_runtime_binding_preserves_unrelated_json_escape_bytes(self):
        for directory, clock in [(self.base, 10), (self.head, 20)]:
            reply = {'result': {'fetched': clock, 'description': 'path/a'}}
            frame = {'jsonrpc': '2.0', 'id': 1, 'result': {
                'structuredContent': reply,
                'content': [{'type': 'text', 'text': json.dumps(reply)}]}}
            output = json.dumps(frame)
            if directory == self.head:
                # Change only the text string's outer slash spelling.
                output = 'path\\/a'.join(output.rsplit('path/a', 1))
            self.write(directory, 'case/mcp-escape.yaml/step/0', output.encode(), surface='mcp')
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr + result.stdout)

    def test_declared_record_directory_does_not_bind_neighbor_paths(self):
        for directory, suffix in [(self.base, 'base'), (self.head, 'head')]:
            root = '/tmp/' + suffix
            self.write(directory, 'case/record.yaml/step/0',
                       args=['get', 'gene', '--record', root + '/record'], root=root)
            args = ['get', 'gene', '--replay', root + '/unrelated']
            digest = hashlib.sha256(json.dumps(args, separators=(',', ':')).encode()).hexdigest()
            self.write(directory, 'trial/cache/replay/' + digest + '/0', args=args,
                       root='/tmp/other-' + suffix)
            # Declare the record directory in the same trial owner too.
            self.write(directory, 'trial/cache/replay/' + 'd' * 64 + '/0',
                       args=['get', 'gene', '--record', root + '/record'], root=root)
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr + result.stdout)

    def test_record_hash_alias_preserves_trial_occurrence(self):
        for directory, suffix in [(self.base, 'base'), (self.head, 'head')]:
            root = '/tmp/' + suffix
            args = ['get', 'gene', '--record', root + '/record']
            stable = [a.replace(root, '<root>') for a in args]
            digest = hashlib.sha256(json.dumps(stable, separators=(',', ':')).encode()).hexdigest()
            self.write(directory, 'trial/cache/replay/' + digest + '/0', args=args, root=root)
            args = ['get', 'gene', '--replay', root + '/record']
            digest = hashlib.sha256(json.dumps(args, separators=(',', ':')).encode()).hexdigest()
            occurrence = '0' if suffix == 'base' else '1'
            self.write(directory, 'trial/cache/replay/' + digest + '/' + occurrence,
                       args=args, root='/tmp/other-' + suffix)
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr + result.stdout)

    def test_section_continuation_preserves_every_reference(self):
        for directory, token in [(self.base, "base-token"), (self.head, "head-token")]:
            reply = {"schema": "biomcp.trial-cards/9", "query": {"input": ["NCT03511664"]},
                     "result": {"items": [{"sections": {"locations": {
                         "state": "data", "source": "ctgov", "data": {"continuation": {
                             "input": ["NCT03511664"], "section": ["locations"],
                             "limit": 10, "offset": 10, "pool": token}}}}}]},
                     "next": [{"kind": "page", "args": ["get", "trial", "NCT03511664",
                                                          "locations", "--pool", token]}]}
            self.write(directory, "case/section.yaml/step/0", json.dumps(reply).encode())
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        path = next(self.head.glob("*.json"))
        record = json.loads(path.read_text())
        reply = json.loads(bytes(record["executions"][0]["stdout"]))
        reply["result"]["items"][0]["sections"]["locations"]["data"]["continuation"]["pool"] = "broken"
        record["executions"][0]["stdout"] = list(json.dumps(reply).encode())
        path.write_text(json.dumps(record))
        self.assertNotEqual(self.compare().returncode, 0)

    def test_opaque_next_tokens_align_later_requests_and_keep_exit_visible(self):
        for directory, suffix in [(self.base, "base"), (self.head, "head")]:
            root, token = "/tmp/" + suffix, suffix + "-token"
            reply = {"schema": "biomcp.article-evidence/6", "query": {"citing": "33123537",
                     "cited": "10.1038/nature05933"}, "next": [{"kind": "page", "args": [
                         "article", "citation-evidence", "33123537", "10.1038/nature05933",
                         "--pool", token]}]}
            args = ["article", "citation-evidence", "33123537", "10.1038/nature05933",
                    "--record", root + "/record"]
            digest = hashlib.sha256(json.dumps([a.replace(root, "<root>") for a in args],
                                     separators=(",", ":")).encode()).hexdigest()
            self.write(directory, "trial/evidence/continuation/" + digest + "/0",
                       json.dumps(reply).encode(), args=args, root=root)
            args = ["article", "citation-evidence", "33123537", "10.1038/nature05933",
                    "--pool", token, "--replay", root + "/record"]
            digest = hashlib.sha256(json.dumps([a.replace(root, "<root>") for a in args],
                                     separators=(",", ":")).encode()).hexdigest()
            reply["query"]["pool"] = token
            reply.pop("next")
            self.write(directory, "trial/evidence/continuation/" + digest + "/0",
                       json.dumps(reply).encode(), args=args, root=root)
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        for path in self.head.glob("*.json"):
            record = json.loads(path.read_text())
            if "--pool" in record["executions"][0]["args"]:
                record["executions"][0]["exit"] = 4
                path.write_text(json.dumps(record))
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("/0/exit", result.stdout)
        self.assertNotIn("missing", result.stdout)

    def test_aliased_reply_declarations_align_the_next_request(self):
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            parent, child = "/tmp/" + side, "/tmp/child-" + side
            first, second = side + "-first", side + "-second"
            for stage in range(3):
                root = parent if stage == 0 else child
                args = ["article", "citation-evidence", "33123537", "10.1038/nature05933"]
                if stage == 2:
                    args += ["--pool", second]
                args += ["--record" if stage == 0 else "--replay", parent + "/record"]
                stable = [arg.replace(root, "<root>") for arg in args]
                digest = hashlib.sha256(json.dumps(stable, separators=(",", ":")).encode()).hexdigest()
                reply = {"schema": "biomcp.article-evidence/6", "query": {}}
                if stage < 2:
                    reply["next"] = [{"kind": "page", "args": ["article", "citation-evidence",
                        "33123537", "10.1038/nature05933", "--pool", first if stage == 0 else second]}]
                else:
                    reply["query"]["pool"] = second
                self.write(directory, "trial/evidence/chain/" + digest + "/0",
                           json.dumps(reply).encode(), args=args, root=root)
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_snapshot_sections_bind_only_their_declared_token_slots(self):
        for directory, token in [(self.base, "base-token"), (self.head, "head-token")]:
            reply = {"schema": "biomcp.phenotype-card/5", "query": {"id": "HP:0000001"},
                     "next": [{"kind": "section", "args": ["get", "phenotype", "HP:0000001",
                                                               "synonyms", "--snapshot", token]}]}
            self.write(directory, "case/phenotype.yaml/step/0", json.dumps(reply).encode())
            self.write(directory, "case/phenotype.yaml/step/1", json.dumps({
                "schema": "biomcp.phenotype-card/5", "query": {"snapshot": token}}).encode(),
                args=["get", "phenotype", "HP:0000001", "synonyms", "--snapshot", token])
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        for directory, token in [(self.base, "base-token"), (self.head, "head-token")]:
            self.write(directory, "case/phenotype.yaml/step/2", json.dumps({
                "result": {"description": token}}).encode())
        self.assertEqual(self.compare().returncode, 1)

    def test_nested_trial_names_keep_independent_runtime_owners(self):
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            for name, clock in [("first", 10), ("second", 11)]:
                self.write(directory, "trial/article-evidence/evidence/" + name + "/" + "a" * 64 + "/0",
                           json.dumps({"result": {"fetched": clock if side == "base" else 20},
                               "page": {"pool": {"id": "shared" if side == "base" else name,
                                                  "created": 10, "expires": 20, "size": 1}}}).encode(),
                           root="/tmp/" + side + "/" + name)
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def pgx_capture(self, directory, token, mutation=None):
        args = ['get', 'pgx', 'annotations', '--gene', 'DPYD', '--kind', 'clinical',
                '--section-limit', '1', '--section-offset', '1', '--snapshot', token,
                '--source', 'clinpgx']
        reply = {'schema': 'biomcp.pgx-annotations/5',
                 'query': {'gene': 'DPYD', 'source': 'clinpgx', 'section': 'annotations'},
                 'next': [{'kind': 'page', 'args': args}]}
        if mutation == 'schema':
            reply['schema'] = 'biomcp.pgx-card/4'
        if mutation == 'kind':
            reply['next'][0]['kind'] = 'section'
        if mutation == 'source-nesting':
            reply = {'source': reply}
        self.write(directory, 'case/pgx.yaml/step/0', json.dumps(reply).encode())
        reference = 'broken' if mutation == 'reference' else token
        self.write(directory, 'case/pgx.yaml/step/1', json.dumps({
            'schema': 'biomcp.pgx-annotations/5', 'query': {'snapshot': reference},
            'result': {'lifetime': 86401 if mutation == 'lifetime' else 86400},
        }).encode(), args=args[:12] + [reference] + args[13:])
        markdown = '**page:**\n\n```sh\nbiomcp ' + ' '.join(args) + '\n```\n'
        if mutation == 'markdown-reference':
            markdown = markdown.replace(token, 'broken')
        if mutation == 'source-text':
            markdown += '\nSource says --snapshot ' + token + '\n'
        self.write(directory, 'case/pgx.yaml/step/2', markdown.encode())
        if mutation == 'source-field':
            self.write(directory, 'case/pgx.yaml/step/3', json.dumps({
                'source': {'snapshot': token, 'next': [{'kind': 'page', 'args': args}]}
            }).encode())
        if mutation == 'rotated':
            reply['next'][0]['args'][12] = token + '-rotated'
            self.write(directory, 'case/pgx.yaml/step/3', json.dumps(reply).encode())

    def test_pgx_snapshot_declaration_binds_json_argv_and_markdown_references(self):
        self.pgx_capture(self.base, 'base-token')
        self.pgx_capture(self.head, 'head-token')
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_pgx_snapshot_binding_keeps_reference_lifetime_and_source_changes(self):
        for mutation in ['reference', 'markdown-reference', 'lifetime', 'source-text',
                         'source-field', 'rotated', 'schema', 'kind', 'source-nesting']:
            with self.subTest(mutation=mutation):
                for directory in (self.base, self.head):
                    for path in directory.glob('*.json'):
                        path.unlink()
                # Source collisions differ on both sides, even after a valid declaration.
                shared = mutation if mutation in ('source-text', 'source-field', 'schema',
                                                  'kind', 'source-nesting') else None
                self.pgx_capture(self.base, 'base-token', shared)
                self.pgx_capture(self.head, 'head-token', mutation)
                result = self.compare()
                self.assertNotEqual(result.returncode, 0, result.stdout)

    def test_pgx_declared_snapshot_pairs_hashed_mcp_invocations(self):
        for directory, token in [(self.base, 'base-token'), (self.head, 'head-token')]:
            args = ['get', 'pgx', 'annotations', '--gene', 'DPYD', '--kind', 'clinical',
                    '--section-limit', '1', '--section-offset', '1', '--snapshot', token,
                    '--source', 'clinpgx']
            self.write(directory, 'trial/pgx/' + 'a' * 64 + '/0', json.dumps({
                'schema': 'biomcp.pgx-annotations/5',
                'next': [{'kind': 'page', 'args': args}]}).encode())
            frame = {'jsonrpc': '2.0', 'method': 'tools/call',
                     'params': {'name': 'biomcp', 'arguments': {'args': args}}}
            command = ['serve', '<requests>', json.dumps(frame)]
            digest = hashlib.sha256(json.dumps(command, separators=(',', ':')).encode()).hexdigest()
            self.write(directory, 'trial/pgx/' + digest + '/0', args=command, surface='mcp')
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def owned_replay_capture(self, *, suffix="recordings", mutation=None):
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            root = "/tmp/replies-" + side
            if suffix == "all":
                for step, name in enumerate(["first", "second"]):
                    self.write(directory, "case/owned.yaml/step/" + str(step),
                               args=["help", "--record", root + "/" + name], root=root,
                               code=4 if side == "head" and mutation == "failed-input" else 0)
            args = ["help", "--replay", root + "/" + suffix]
            extra = dict(root="/tmp/runtime-" + side, replay_owner_root=root)
            if side == "head":
                if mutation == "root":
                    extra["replay_owner_root"] += "/other"
                elif mutation == "noncanonical":
                    extra["replay_owner_root"] += "/../" + root.rsplit("/", 1)[1]
                elif mutation == "missing":
                    extra.pop("replay_owner_root")
                elif mutation == "delimiter":
                    args.insert(1, "--")
                elif mutation == "duplicate":
                    args += ["--replay", root + "/" + suffix]
                elif mutation == "suffix":
                    args[-1] += "/other"
                elif mutation == "other-owner":
                    path = directory / (hashlib.sha256(b"case/owned.yaml/step/1").hexdigest() + ".json")
                    path.unlink()
                    self.write(directory, "case/unrelated.yaml/step/1",
                               args=["help", "--record", root + "/second"], root=root)
            self.write(directory, "case/owned.yaml/step/2", args=args, **extra)

    def test_runner_owned_replay_metadata_binds_exact_argv_only(self):
        self.owned_replay_capture()
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        for mutation in ["root", "noncanonical", "missing", "delimiter", "duplicate", "suffix"]:
            with self.subTest(mutation=mutation):
                self.owned_replay_capture(mutation=mutation)
                self.assertNotEqual(self.compare().returncode, 0)
        self.owned_replay_capture()
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            path = directory / (hashlib.sha256(b"case/owned.yaml/step/2").hexdigest() + ".json")
            record = json.loads(path.read_text())
            record["executions"][0]["stdout"] = list(json.dumps({"source": "/tmp/replies-" + side}).encode())
            path.write_text(json.dumps(record))
        self.assertEqual(self.compare().returncode, 1)

    def test_replay_owner_root_cannot_rebind_within_one_capture_owner(self):
        self.owned_replay_capture()
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            root = "/tmp/replies-" + side + ("-other" if side == "head" else "")
            self.write(directory, "case/owned.yaml/step/3",
                       args=["help", "--replay", root + "/recordings"],
                       root="/tmp/runtime-" + side, replay_owner_root=root)
        self.assertNotEqual(self.compare().returncode, 0)

    def test_nested_case_name_cannot_inherit_another_execution_owner(self):
        self.owned_replay_capture()
        for directory, side in [(self.base, "base"), (self.head, "head")]:
            self.write(directory, "case/owned.yaml/step/2/execution/0/step/0",
                       args=["help", "--replay", "/tmp/replies-" + side + "/recordings"],
                       root="/tmp/unrelated-" + side)
        self.assertEqual(self.compare().returncode, 1)

    def test_combined_replay_metadata_requires_successful_same_owner_inputs(self):
        self.owned_replay_capture(suffix="all")
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        for mutation in ["failed-input", "other-owner"]:
            with self.subTest(mutation=mutation):
                self.owned_replay_capture(suffix="all", mutation=mutation)
                self.assertNotEqual(self.compare().returncode, 0)

    def test_operator_map_cannot_declare_external_replay_ownership(self):
        key = "case/unrelated.yaml/step/0"
        owners = Path(self.directory.name, "owners.json")
        source = Path(self.directory.name, "source.rs")
        source.write_text("// This comment proves no capture ownership.\n")
        binding = {"file": source.name, "sha256": hashlib.sha256(source.read_bytes()).hexdigest()}
        declarations = {}
        for side, directory in [("base", self.base), ("head", self.head)]:
            replay = "/tmp/unrelated-" + side + "/recordings"
            self.write(directory, key, args=["get", "trial", "--replay", replay],
                       root="/tmp/command-" + side)
            capture = next(directory.glob("*.json"))
            declarations[side] = [{"key": key, "execution": 0,
                "capture": {"file": capture.name,
                            "sha256": hashlib.sha256(capture.read_bytes()).hexdigest()},
                "replay_root": replay, "source": dict(binding, helper="checked_steps"),
                "helper_source": binding}]
        self.assertEqual(self.compare().returncode, 1)
        owners.write_text(json.dumps(declarations))
        result = self.compare(owners)
        self.assertEqual(result.returncode, 2, result.stderr + result.stdout)
        self.assertIn("usage:", result.stderr)

    def test_invalid_capture_keys_do_not_invent_an_owner(self):
        for key in ["unknown/name/0", "case/name/step/no", "case//step/0",
                    "trial/evidence/name/hash/0", "trial//" + "a" * 64 + "/0",
                    "trial/" + "a" * 64 + "/0"]:
            with self.subTest(key=key):
                for directory in (self.base, self.head):
                    for path in directory.glob("*.json"):
                        path.unlink()
                    self.write(directory, key)
                result = self.compare()
                self.assertEqual(result.returncode, 2, result.stderr + result.stdout)
                self.assertIn("invalid transcript key", result.stderr)

    def test_uncaptured_trials_are_disclosed(self):
        for directory in [self.base, self.head]:
            self.write(directory, "case/help.yaml/step/0")
            (directory / "lifecycle.json").write_text(json.dumps(
                dict(kind="trial", name="mcp/broken-output", uncaptured=True)))
        result = self.compare()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("mcp/broken-output", result.stderr)


if __name__ == "__main__":
    unittest.main()
