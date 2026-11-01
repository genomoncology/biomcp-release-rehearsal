"""Security boundaries exercised through the real scanner and audit commands."""
import io
import json
import os
from pathlib import Path
from release.smoke import environment
import secrets
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]


class Screening(unittest.TestCase):
    def test_history_scan_uses_isolated_git_location_and_fails_closed_without_git(self):
        from release.secrets import scan
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            repo, tools, missing = [root / name for name in ["repo", "usr/local/bin", "missing"]]
            for directory in [repo, tools, missing]:
                directory.mkdir(parents=True)
            (tools / "git").symlink_to(shutil.which("git"))
            env = {**environment(root), "PATH": str(tools), "GIT_CONFIG_NOSYSTEM": "1"}
            for args in [["init", "--quiet"], ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                          "commit", "--quiet", "--allow-empty", "-m", "Initial fixture"]]:
                subprocess.run(["git", "-C", str(repo), *args], env=env, check=True, capture_output=True)
            original_search = os.get_exec_path
            def isolated_search(env=None):
                # Relocate fixed system directories into owned scratch. Real
                # subprocess lookup and Git still execute; no host tools change.
                return [str(tools) if entry == "/usr/local/bin" else str(missing)
                        for entry in original_search(env)]
            with patch("os.defpath", str(missing)), patch("os.get_exec_path", side_effect=isolated_search), patch.dict(os.environ, {"PATH": str(missing)}):
                self.assertEqual(scan("history", repo, ROOT / "target/security-tools"), 0)
                (tools / "git").unlink()
                with self.assertRaises(FileNotFoundError):
                    scan("history", repo, ROOT / "target/security-tools")

    def test_history_scan_excludes_detached_siblings_but_keeps_candidate_and_named_refs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            repo, sibling = root / "repo", root / "sibling"
            repo.mkdir()
            env = {**environment(root), "PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1"}
            def git(*args):
                return subprocess.check_output(["git", "-C", str(repo), *args], env=env,
                                               stderr=subprocess.DEVNULL).decode().strip()
            git("init", "--quiet")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            git("commit", "--quiet", "--allow-empty", "-m", "Initial fixture")
            clean = git("rev-parse", "HEAD")
            git("worktree", "add", "--quiet", "--detach", str(sibling))
            token = "ghp_" + secrets.token_hex(18)
            (sibling / "credential.txt").write_text("access_token = " + token + "\n")
            git("-C", str(sibling), "add", ".")
            git("-C", str(sibling), "commit", "--quiet", "-m", "Add sibling fixture")
            secret = git("-C", str(sibling), "rev-parse", "HEAD")
            def screen(expected):
                result = subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), "history", str(repo)],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, "history scope changed")
                self.assertNotIn(token, result.stdout + result.stderr)
            screen(0)
            git("branch", "reviewed-ref", secret)
            screen(1)
            git("branch", "-D", "reviewed-ref")
            # The secret remains in candidate history after its file is removed.
            git("checkout", "--detach", secret)
            git("rm", "credential.txt")
            git("commit", "--quiet", "-m", "Remove candidate fixture")
            screen(1)
            # Replacement refs must not hide the candidate's actual ancestry.
            git("replace", "HEAD", clean)
            screen(1)

    def test_scanner_rejects_historical_and_nested_artifact_secrets_without_disclosure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            # Construct only in owned scratch; no token or secret fixture enters Git.
            token = "ghp_" + "".join(secrets.choice("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789") for _ in range(36))
            repo = root / "repo"
            repo.mkdir()
            env = {"PATH": os.defpath, "HOME": str(root), "GIT_CONFIG_NOSYSTEM": "1"}
            def git(*args):
                subprocess.run(["git", "-C", str(repo), *args], env=env, check=True, capture_output=True)
            git("init", "--quiet")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            (repo / "credential.txt").write_text("access_token = " + token + "\n")
            git("add", ".")
            git("commit", "--quiet", "-m", "Add fixture")
            git("rm", "credential.txt")
            git("commit", "--quiet", "-m", "Remove fixture")
            artifacts = root / "artifacts"
            artifacts.mkdir()
            inner = root / "payload.whl"
            with zipfile.ZipFile(inner, "w") as archive:
                archive.writestr("credential.txt", "access_token = " + token + "\n")
            with tarfile.open(artifacts / "container.tar", "w") as archive:
                archive.add(inner, arcname="layer/payload.whl")
            for mode, target in [("history", repo), ("artifacts", artifacts)]:
                result = subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), mode, str(target)],
                                        cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
                self.assertEqual(result.returncode, 1, "planted fixture must fail secret screening")
                self.assertTrue(token not in result.stdout + result.stderr, "scanner disclosed matched text")
                self.assertTrue("access_token =" not in result.stdout + result.stderr, "scanner disclosed a source line")
                findings = [json.loads(line) for line in result.stdout.splitlines()]
                self.assertTrue(findings)
                self.assertTrue(all(set(row) == {"file", "rule"} for row in findings))
                self.assertIn("github-pat", {row["rule"] for row in findings})

    def test_scanner_covers_binary_members_and_merge_only_history(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            token = "ghp_" + secrets.token_hex(18)
            payload = b"MZ" + bytes(128) + ("access_token = " + token + "\n").encode()
            artifacts = root / "artifacts"
            artifacts.mkdir()
            env = {**environment(root), "PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1"}
            def screen(mode, target):
                result = subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), mode, str(target)],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 1, "planted binary or merge fixture must fail screening")
                self.assertNotIn(token, result.stdout + result.stderr)
                for row in map(json.loads, result.stdout.splitlines()):
                    self.assertEqual(set(row), {"file", "rule", "level"} if row.get("level") == "info" else {"file", "rule"})
                return result.stdout
            for name in ["biomcp.exe", "package.dist-info/release.json"]:
                with zipfile.ZipFile(artifacts / "candidate.whl", "w") as archive:
                    archive.writestr(name, payload)
                self.assertIn(name, screen("artifacts", artifacts))
            repo = root / "repo"
            repo.mkdir()
            def git(*args, data=None):
                return subprocess.run(["git", "-C", str(repo), *args], input=data, env=env,
                                      check=True, capture_output=True).stdout.decode().strip()
            git("init", "--quiet")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            git("commit", "--allow-empty", "--quiet", "-m", "Initial fixture")
            first = git("rev-parse", "HEAD")
            git("commit", "--allow-empty", "--quiet", "-m", "Second fixture")
            second = git("rev-parse", "HEAD")
            (repo / "merge-only.txt").write_text("access_token = " + token + "\n")
            (repo / "historical-opaque.bin").write_bytes(b"PK\x03\x04synthetic prefix\n" + ("access_token = " + token + "\n").encode())
            with zipfile.ZipFile(repo / "historical-valid.bin", "w", zipfile.ZIP_DEFLATED) as archive:
                archive.writestr("credential.exe", payload)
            buffer = io.BytesIO()
            with zipfile.ZipFile(buffer, "w") as archive:
                archive.writestr("credential.exe", payload)
            malformed = bytearray(buffer.getvalue())
            malformed[30] ^= 1  # Local and central filenames intentionally disagree.
            (repo / "historical-structure.bin").write_bytes(malformed)
            encrypted = bytearray(buffer.getvalue())
            encrypted[6] |= 1
            encrypted[encrypted.index(b"PK\x01\x02") + 8] |= 1
            (repo / "historical-encrypted.bin").write_bytes(encrypted)
            directory_size = next((ROOT / "tests/replies/article/supplements/synthetic-bound-directory-size").rglob("*.bin")).read_bytes()
            (repo / "historical-directory-size.bin").write_bytes(directory_size + ("\naccess_token = " + token + "\n").encode())
            git("add", ".")
            tree = git("write-tree")
            merged = git("commit-tree", tree, "-p", first, "-p", second, data=b"Merge fixture\n")
            git("update-ref", "HEAD", merged)
            findings = screen("history", repo)
            self.assertIn("historical-opaque.bin", findings)
            self.assertIn("historical-structure.bin", findings)
            self.assertTrue(any(row["file"] == "historical-directory-size.bin" and row["rule"] == "opaque-history-blob-decoded-coverage-unavailable" for row in map(json.loads, findings.splitlines())))
            self.assertTrue(any(row["file"] == "historical-directory-size.bin" and row["rule"] == "github-pat" for row in map(json.loads, findings.splitlines())))
            self.assertTrue(any(row["file"].endswith("historical-encrypted.bin") and row["rule"] == "opaque-history-blob-decoded-coverage-unavailable" for row in map(json.loads, findings.splitlines())))
            self.assertIn("historical-valid.bin!credential.exe", findings)
            with zipfile.ZipFile(repo / "nested-declared.bin", "w") as archive:
                archive.writestr("corrupt.zip", b"PK\x03\x04malformed declared archive")
            git("add", ".")
            git("commit", "--quiet", "-m", "Add declared archive rejection fixture")
            rejected = subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), "history", str(repo)],
                                      cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(rejected.returncode, 2, "outer opaque history policy must not hide declared archive corruption")
            self.assertNotIn(token, rejected.stdout + rejected.stderr)

    def test_scanner_refuses_unreadable_artifacts_and_never_follows_links(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            artifacts = root / "artifacts"
            artifacts.mkdir()
            env = {**environment(root), "PATH": os.defpath}
            def screen():
                return subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), "artifacts", str(artifacts)],
                                      cwd=ROOT, env=env, capture_output=True, text=True)
            candidate = artifacts / "candidate.zip"
            candidate.write_bytes(b"PK\x03\x04broken archive")
            corrupt = screen()
            self.assertEqual(corrupt.returncode, 2, "corrupt archive must fail closed")
            self.assertEqual(corrupt.stderr, "Secret screening failed; diagnostic output was suppressed\n")
            candidate.write_bytes(next((ROOT / "tests/replies/article/supplements/synthetic-bound-directory-size").rglob("*.bin")).read_bytes())
            self.assertEqual(screen().returncode, 2, "malformed directory size in a staged archive must fail closed")
            with zipfile.ZipFile(candidate, "w") as archive:
                archive.writestr("../escape.txt", "fixture")
            self.assertNotEqual(screen().returncode, 0, "escaping member must fail closed")
            with zipfile.ZipFile(candidate, "w", zipfile.ZIP_DEFLATED) as archive:
                archive.writestr("hidden/", "directory content")
            self.assertNotEqual(screen().returncode, 0, "directory payload must not be skipped")
            with zipfile.ZipFile(candidate, "w") as archive:
                archive.writestr("credential.txt", "fixture")
            encrypted = bytearray(candidate.read_bytes())
            encrypted[6] |= 1
            encrypted[encrypted.index(b"PK\x01\x02") + 8] |= 1
            candidate.write_bytes(encrypted)
            self.assertEqual(screen().returncode, 2, "staged encrypted archive must fail closed")
            candidate.unlink()
            candidate.symlink_to(root / "absent")
            self.assertNotEqual(screen().returncode, 0, "filesystem links must not be skipped")
            candidate.unlink()
            payload = b"fixture"
            for _ in range(18):
                buffer = io.BytesIO()
                with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
                    archive.writestr("nested.zip", payload)
                payload = buffer.getvalue()
            candidate.write_bytes(payload)
            self.assertNotEqual(screen().returncode, 0, "depth limits must fail closed")
            candidate.unlink()
            with tarfile.open(artifacts / "layer.tar", "w") as archive:
                link = tarfile.TarInfo("usr/share/man/fixture.1.gz")
                link.type, link.linkname = tarfile.SYMTYPE, "/outside/fixture"
                archive.addfile(link)
            self.assertEqual(screen().returncode, 0, "container links must be screened without dereferencing")

    def test_history_exceptions_require_the_exact_blob_rule_line_and_commit(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            tools, repo = root / "tools", root / "repo"
            tools.mkdir()
            repo.mkdir()
            for name in ["secrets.py", "smoke.py", "toolchain.py", "toolchain.json"]:
                shutil.copy2(ROOT / "release" / name, tools / name)
            env = {**environment(root), "PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1"}
            def git(*args):
                return subprocess.check_output(["git", "-C", str(repo), *args], env=env, stderr=subprocess.DEVNULL).decode().strip()
            git("init", "--quiet")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            (repo / "opaque.bin").write_bytes(b"PK\x03\x04synthetic opaque fixture")
            git("add", ".")
            git("commit", "--quiet", "-m", "Initial fixture")
            empty = git("rev-parse", "HEAD")
            token = "ghp_" + secrets.token_hex(18)
            path = "tests/replies/credential.json"
            file = repo / path
            file.parent.mkdir(parents=True)
            file.write_text('api_key = "' + token + '"\n')
            git("add", ".")
            git("commit", "--quiet", "-m", "Add fixture")
            base = {"commit": git("rev-parse", "HEAD"), "blob": git("rev-parse", "HEAD:" + path),
                    "file": path, "line": 1, "reason": "Synthetic exact-binding refusal fixture."}
            entries = [{**base, "rule": rule} for rule in ["github-pat", "generic-api-key"]]
            ignore = tools / "secrets.ignore.jsonl"
            def screen(records, expected, code=None, row=1):
                ignore.write_text("".join(json.dumps(row) + "\n" for row in records))
                result = subprocess.run([sys.executable, str(tools / "secrets.py"), "history", str(repo),
                                         "--cache", str(ROOT / "target/security-tools")],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, "exact history binding changed behavior")
                self.assertNotIn(token, result.stdout + result.stderr)
                self.assertNotIn('api_key =', result.stdout + result.stderr)
                if expected == 0:
                    informational = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
                    self.assertTrue(informational)
                    self.assertTrue(all(item.get("level") == "info" for item in informational))
                if code:
                    self.assertEqual(result.stderr, json.dumps({"code": code, "row": row}) + "\n")
            screen(entries, 0)
            sibling = root / "sibling"
            git("worktree", "add", "--quiet", "--detach", str(sibling), base["commit"])
            git("-C", str(sibling), "commit", "--quiet", "--allow-empty", "-m", "Sibling binding fixture")
            sibling_commit = git("-C", str(sibling), "rev-parse", "HEAD")
            git("branch", "side-only-exception", sibling_commit)
            screen([entries[0], {**entries[1], "commit": sibling_commit}], 2, "history-ignore-unreachable-commit", row=2)
            screen([{**r, "line": 2} for r in entries], 2, "history-ignore-wrong-line")
            screen([{**r, "rule": "unrelated-rule"} for r in entries], 1)
            screen([{**r, "commit": empty} for r in entries], 2, "history-ignore-wrong-blob")
            file.write_text(file.read_text() + "changed blob\n")
            git("add", ".")
            git("commit", "--quiet", "-m", "Change the same path")
            screen(entries, 1)

    def test_working_tree_patterns_do_not_allow_other_fields_or_accessions(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            artifacts = root / "artifacts"
            path = artifacts / "tests/replies/trial/fixture.json"
            path.parent.mkdir(parents=True)
            cursor = secrets.token_urlsafe(29)
            digest = secrets.token_hex(32)
            env = {**environment(root), "PATH": os.defpath}
            def screen(expected):
                result = subprocess.run([sys.executable, str(ROOT / "release/secrets.py"), "artifacts", str(artifacts)],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, "fixture regex changed the scanner boundary")
                return result
            path.write_text(json.dumps({"nextPageToken": cursor, "request_key": digest}))
            screen(0)
            token = "ghp_" + secrets.token_hex(18)
            path.write_text(json.dumps({"nextPageToken": cursor, "request_key": digest, "api_key": token}))
            rejected = screen(1)
            self.assertNotIn(token, rejected.stdout + rejected.stderr)
            path.unlink()
            cell = artifacts / "tests/fixtures/cell-line/native/portable-cellosaurus-cell-CVCL_0004.json"
            cell.parent.mkdir(parents=True)
            cell.write_text(json.dumps({"xref-list": [{"database": "Fixture", "accession": token}]}))
            rejected = screen(1)
            self.assertNotIn(token, rejected.stdout + rejected.stderr)

    def test_audit_denies_an_unknown_git_source_after_successful_resolution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            dependency = root / "dependency"
            dependency.mkdir()
            (dependency / "src").mkdir()
            (dependency / "src/lib.rs").write_text("pub fn fixture() {}\n")
            (dependency / "Cargo.toml").write_text('[package]\nname="source-fixture"\nversion="0.1.0"\nedition="2021"\nlicense="MIT"\n')
            env = {**environment(root), **{name: os.environ[name] for name in ["PATH", "CARGO_HOME", "RUSTUP_HOME"] if name in os.environ}, "GIT_CONFIG_COUNT": "1", "GIT_CONFIG_KEY_0": "url." + dependency.as_uri() + ".insteadOf",
                   "GIT_CONFIG_VALUE_0": "https://unknown.example.invalid/dependency",
                   "CARGO_HOME": os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
                   "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")), "CARGO_NET_GIT_FETCH_WITH_CLI": "true", "GIT_TERMINAL_PROMPT": "0"}
            for args in [["init", "--quiet"], ["add", "."], ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "Add fixture"]]:
                subprocess.run(["git", "-C", str(dependency), *args], env=env, check=True, capture_output=True)
            project = root / "project"
            project.mkdir()
            (project / "src").mkdir()
            (project / "src/lib.rs").touch()
            (project / "Cargo.toml").write_text('[package]\nname="audit-fixture"\nversion="0.1.0"\nedition="2021"\nlicense="MIT"\n[dependencies]\nsource-fixture={git="https://unknown.example.invalid/dependency"}\n')
            for name in ["Makefile", "deny.toml"]:
                (project / name).write_bytes((ROOT / name).read_bytes())
            resolved = subprocess.run(["cargo", "generate-lockfile"], cwd=project, env=env, capture_output=True, text=True, timeout=120)
            self.assertEqual(resolved.returncode, 0, resolved.stderr)
            result = subprocess.run(["make", "audit"], cwd=project, env=env, capture_output=True, text=True, timeout=120)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unknown.example.invalid/dependency", result.stdout + result.stderr)
            self.assertIn("source", result.stdout + result.stderr)
            self.assertRegex(result.stdout + result.stderr, "(?i)(unapproved|not explicitly allowed|not allowed|unknown-git)")


if __name__ == "__main__":
    unittest.main()
