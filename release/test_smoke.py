"""Installed commands own canonical temporary space, including on failure."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


class InstalledTemporarySpace(unittest.TestCase):
    def test_actual_installed_cli_refuses_a_different_candidate_commit(self):
        # Use the selected built executable through the same CLI checker as every installer.
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / ('biomcp.exe' if os.name == 'nt' else 'biomcp')
            shutil.copy2(os.environ['BIOMCP_INSTALLED_BINARY'], binary)
            script = """
import hashlib, json, pathlib, sys
sys.path.insert(0, 'release')
from gate import cli_checks
binary = pathlib.Path(sys.argv[1])
import subprocess
verbose = subprocess.check_output([str(binary), 'version', '--verbose'], text=True)
revision = next(line.removeprefix('Commit: ') for line in verbose.splitlines() if line.startswith('Commit: '))
record = {'version': verbose.splitlines()[0].removeprefix('biomcp '),
          'revision': revision, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest()}
rows = cli_checks(binary, record)
assert [row['command'] for row in rows] == [['version'], ['version', '--verbose'], ['health', '--offline']]
record['revision'] = ('b' if revision == 'a' * 40 else 'a') * 40
try:
    cli_checks(binary, record)
except ValueError as error:
    assert str(error) == 'installed CLI commit differs from candidate', error
else:
    raise AssertionError('wrong candidate commit passed')
"""
            result = subprocess.run([sys.executable, '-c', script, str(binary)],
                                    capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    @unittest.skipUnless(sys.platform == 'darwin' and shutil.which('brew'), 'staged formula installation needs macOS Homebrew')
    def test_staged_formula_installs_the_selected_candidate(self):
        import hashlib
        from release.artifacts import binary_archive, platform_tag
        from release.workflow import formula
        binary = Path(os.environ['BIOMCP_INSTALLED_BINARY']).resolve()
        verbose = subprocess.check_output([str(binary), 'version', '--verbose'], text=True)
        revision = next(line.removeprefix('Commit: ') for line in verbose.splitlines() if line.startswith('Commit: '))
        version = verbose.splitlines()[0].removeprefix('biomcp ')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            staged = root / 'staged'
            staged.mkdir()
            for arch in ('arm64', 'x86_64'):
                binary_archive(staged, binary, 'macosx_11_0_' + arch)
            (staged / 'candidate.json').write_text(json.dumps({'version': version, 'revision': revision}))
            formula(staged, version)
            record = {'version': version, 'revision': revision, 'platform': platform_tag(False),
                      'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest()}
            (root / 'release.json').write_text(json.dumps(record))
            command = [sys.executable, 'release/gate.py', '--artifacts', str(root),
                       '--scratch', str(root / 'check'), '--staged-formula', str(staged / 'biomcp.rb')]
            result = subprocess.run(command, capture_output=True, text=True, timeout=300)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            rows = json.loads((root / 'check/report.json').read_text())
            self.assertEqual([row['command'] for row in rows], [['version'], ['version', '--verbose'], ['health', '--offline']])
            self.assertTrue(all(row['artifact'] == 'staged-homebrew-formula' for row in rows))
            self.assertTrue(all(row['revision'] == revision for row in rows))

    def test_installed_alphagenome_uses_selected_binary_without_checkout(self):
        from release.smoke import environment
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary = root / ("biomcp.exe" if os.name == "nt" else "biomcp")
            runner = root / ("case-runner.exe" if os.name == "nt" else "case-runner")
            shutil.copy2(os.environ["BIOMCP_INSTALLED_BINARY"], binary)
            shutil.copy2(os.environ["BIOMCP_CASE_RUNNER"], runner)
            env = environment(root)
            env["BIOMCP_TEST_EXECUTABLE"] = str(binary)
            command = [str(runner), "variant-alphagenome/alphagenome-native-groups-local-tls", "--exact"]
            env["BIOMCP_TEST_FIXTURES"] = str(Path("tests").resolve())
            result = subprocess.run(command, env=env, cwd=root, capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("variant-alphagenome/alphagenome-native-groups-local-tls", result.stdout)
            self.assertIn("1 passed", result.stdout)
            invalid = subprocess.run(command + ["--invalid-option"], env=env, cwd=root,
                                     capture_output=True, text=True, timeout=120)
            self.assertNotEqual(invalid.returncode, 0)
            if os.name != "nt":
                # Keep the actual CLI but break only its transport through a closed local proxy.
                config = root / "broken-network.json"
                config.write_text(json.dumps({"proxy": "http://127.0.0.1:0",
                                               "keys": {"alphagenome_api_key": "synthetic"}}))
                broken = root / "broken-transport"
                broken.write_text(f"#!{sys.executable}\nimport os, sys\nos.execv({str(binary)!r}, [{str(binary)!r}] + sys.argv[1:] + ['--config', {str(config)!r}])\n")
                broken.chmod(0o755)
                env["BIOMCP_TEST_EXECUTABLE"] = str(broken)
                result = subprocess.run(command, env=env, cwd=root, capture_output=True, text=True, timeout=120)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn("variant-alphagenome/alphagenome-native-groups-local-tls", result.stdout)
                self.assertIn("1 failed", result.stdout)

    def test_isolated_installed_commands_and_cleanup(self):
        binary = Path(os.environ["BIOMCP_INSTALLED_BINARY"]).resolve()
        runner = Path(os.environ["BIOMCP_CASE_RUNNER"]).resolve()
        version = os.environ["BIOMCP_INSTALLED_VERSION"]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            installed = root / "installed"
            installed.mkdir()
            shutil.copy2(binary, installed / binary.name)
            shutil.copy2(runner, installed / ("case-runner" + runner.suffix))
            shutil.copy2(runner.with_name("tool-runner" + runner.suffix),
                         installed / ("tool-runner" + runner.suffix))
            fixtures = root / "fixtures"
            shutil.copytree("tests", fixtures)
            provenance = Path("sdlc/planning/slices/gene.md")
            (fixtures / "support" / provenance).parent.mkdir(parents=True)
            shutil.copy2(provenance, fixtures / "support" / provenance)
            parent = root / "job-tmp"
            parent.mkdir()
            sentinel = parent / "keep"
            sentinel.write_text("job-owned")
            script = """
import json, pathlib, sys
from release.smoke import smoke
created = []
sys.addaudithook(lambda event, args: created.append(str(pathlib.Path(args[0]).resolve())) if event == 'tempfile.mkdtemp' else None)
try:
    smoke(*sys.argv[1:])
finally:
    print(json.dumps({'temporary_roots': created}))
"""
            routine = subprocess.check_output([str(runner), "--list"], text=True)
            copied_env = {**os.environ, "BIOMCP_TEST_FIXTURES": str(fixtures)}
            copied = subprocess.check_output([str(installed / ("case-runner" + runner.suffix)), "--list"], env=copied_env, cwd=root, text=True)
            self.assertEqual(routine, copied)
            for configured, fail in [(True, False), (True, True)]:
                with self.subTest(configured=configured, fail=fail):
                    env = {key: value for key, value in os.environ.items() if key not in {"TMPDIR", "TMP", "TEMP"}}
                    if configured:
                        env["TMPDIR"] = str(parent)
                    selected = installed / ("absent" if fail else binary.name)
                    result = subprocess.run([sys.executable, "-c", script, str(selected), str(installed / ("case-runner" + runner.suffix)), str(fixtures), version], env=env, capture_output=True, text=True, timeout=300)
                    self.assertEqual(result.returncode == 0, not fail, result.stdout + result.stderr)
                    roots = json.loads(result.stdout.splitlines()[-1])["temporary_roots"]
                    self.assertEqual(len(roots), 1)
                    self.assertFalse(Path(roots[0]).exists(), "smoke leaked its temporary directory")
                    if configured:
                        self.assertEqual(Path(roots[0]).parent, parent)
                    self.assertEqual(list(parent.iterdir()), [sentinel])
                    self.assertEqual(sentinel.read_text(), "job-owned")


    @unittest.skipIf(os.name == "nt", "the planted executable uses a POSIX shebang")
    def test_installed_schema_change_names_the_case(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary = Path(os.environ["BIOMCP_INSTALLED_BINARY"]).resolve()
            runner = Path(os.environ["BIOMCP_CASE_RUNNER"]).resolve()
            planted = root / "changed-schema"
            planted.write_text(f"#!{sys.executable}\nimport os, sys, json, subprocess\n"
                               "if sys.argv[1:] == ['schema', 'biodata.source/1']:\n"
                               f"    value = json.loads(subprocess.check_output([{str(binary)!r}] + sys.argv[1:]))\n"
                               "    value['$id'] = 'changed.source/1'\n"
                               "    print(json.dumps(value))\n"
                               f"else:\n    os.execv({str(binary)!r}, [{str(binary)!r}] + sys.argv[1:])\n")
            planted.chmod(0o755)
            original_schema = json.loads(subprocess.check_output([str(binary), "schema", "biodata.source/1"]))
            changed_schema = json.loads(subprocess.check_output([str(planted), "schema", "biodata.source/1"]))
            self.assertEqual(original_schema["$id"], "biodata.source/1")
            self.assertEqual(changed_schema["$id"], "changed.source/1")
            restored_schema = {**changed_schema, "$id": original_schema["$id"]}
            self.assertEqual(restored_schema, original_schema)
            print(json.dumps({"planted_script": planted.read_text(), "original_schema": original_schema,
                              "changed_schema": changed_schema}), flush=True)
            fixtures = root / "fixtures"
            shutil.copytree("tests", fixtures)
            provenance = Path("sdlc/planning/slices/gene.md")
            (fixtures / "support" / provenance).parent.mkdir(parents=True)
            shutil.copy2(provenance, fixtures / "support" / provenance)
            case = fixtures / "cases/installed-schema.yaml"
            case.write_text("feature: schema-discovery\nstory: Installed schema identity\nsteps:\n  - run: [schema, biodata.source/1]\n    expect: {'/$id': biodata.source/1}\n")
            script = "from release.smoke import smoke; import sys; smoke(*sys.argv[1:])"
            result = subprocess.run([sys.executable, "-c", script, str(planted), str(runner), str(fixtures),
                                     os.environ["BIOMCP_INSTALLED_VERSION"]], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("installed-schema.yaml", result.stdout + result.stderr)
            diagnostic = "Installed schema identity; step 1: schema validation: raw artifact differs from requested biodata.source/1"
            self.assertIn(diagnostic, result.stdout + result.stderr)
            print(json.dumps({"planted_schema_id": "changed.source/1", "rehearsal_rejected": True,
                              "case": "installed-schema.yaml", "diagnostic": diagnostic}), flush=True)


if __name__ == "__main__":
    unittest.main()
