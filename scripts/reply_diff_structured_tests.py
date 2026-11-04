"""Retain complete structured HTTP captures in the existing reply diagnostic."""
import hashlib
import json

from reply_diff_test_support import ReplyDiffCase


class ReplyStructured(ReplyDiffCase):
    key = "trial/hosted/" + "a" * 64 + "/0"

    def capture(self, directory, executions, key=None):
        key = key or self.key
        path = directory / (hashlib.sha256(key.encode()).hexdigest() + ".json")
        path.write_text(json.dumps({"key": key, "executions": executions}))

    def http(self):
        return dict(args=["get", "gene", "BRAF"], surface="http", http_status=200,
                    supplied_header=True, extra_argument=False,
                    reply={"result": {"status": "complete", "fetched": 1}})

    def concurrent(self):
        return dict(surface="http-concurrent", keyed_reply={"status": "complete"},
                    unkeyed_reply={"status": "failed"})

    def test_structured_replies_and_metadata_remain_literal(self):
        for execution, field, changed in [
                (self.http(), "reply", {"result": {"status": "complete", "fetched": 2}}),
                (self.http(), "supplied_header", False),
                (self.concurrent(), "keyed_reply", {"status": "failed"})]:
            self.capture(self.base, [execution])
            self.capture(self.head, [execution])
            result = self.compare()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("1 records/1 executions", result.stderr)
            self.capture(self.head, [{**execution, field: changed}])
            result = self.compare()
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn("/executions/0/" + field, result.stdout)

    def test_unknown_mixed_and_changed_categories_are_refused(self):
        self.capture(self.base, [self.http()])
        for executions in [
                [{**self.http(), "unknown": True}],
                [{**self.http(), "stdout": [], "exit": 0}],
                [dict(args=["help"], surface="cli", stdout=[], exit=0, unknown=True)],
                [self.http(), self.concurrent()],
                [self.concurrent()]]:
            self.capture(self.head, executions)
            result = self.compare()
            self.assertEqual(result.returncode, 2, result.stderr)

    def test_structured_keys_and_execution_counts_are_compared(self):
        self.capture(self.base, [self.http()])
        self.capture(self.head, [self.http(), self.http()])
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("/executions/1", result.stdout)
        self.capture(self.head, [self.http()])
        added = "trial/hosted/" + "b" * 64 + "/0"
        self.capture(self.head, [self.concurrent()], added)
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn(added + ": / missing in BASE", result.stdout)
        for path in self.head.glob("*.json"):
            path.unlink()
        self.capture(self.head, [self.concurrent()], added)
        result = self.compare()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn(self.key + ": / missing in HEAD", result.stdout)
