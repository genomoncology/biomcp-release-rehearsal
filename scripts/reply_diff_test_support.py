"""One isolated capture fixture serves the reply comparison tests."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("reply_diff.py")


class ReplyDiffCase(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name, "base")
        self.head = Path(self.directory.name, "head")
        self.base.mkdir()
        self.head.mkdir()

    def write(self, directory, key, stdout=b'{"status":"complete"}\n', code=0, **extra):
        execution = dict(args=["help"], surface="cli", exit=code, stdout=list(stdout))
        execution.update(extra)
        path = directory / (hashlib.sha256(key.encode()).hexdigest() + ".json")
        path.write_text(json.dumps(dict(key=key, executions=[execution])))

    def compare(self, owners=None, *, all_fields=False):
        args = [sys.executable, str(SCRIPT), str(self.base), str(self.head)]
        if all_fields:
            args.insert(2, "--all-fields")
        if owners:
            args.append(str(owners))
        return subprocess.run(args,
                              capture_output=True, text=True, check=False)
