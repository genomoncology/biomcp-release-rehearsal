"""Feature install-script: exercise the actual installer with isolated downloads."""
from release.artifacts import binary_archive
from release.local_server import QuietHandler, https_server, server
import hashlib
import io
import os
from pathlib import Path
import subprocess
import shutil
import shlex
import sys
import tarfile
import tempfile
import unittest
import zipfile

from release.test_licenses import write_payloads
from unittest.mock import patch

SCRIPT = Path(__file__).with_name("install.sh").resolve()


def candidate(directory, name, content=b"#!/bin/sh\necho candidate\n", member=None):
    path = directory / name
    if name.endswith(".zip"):
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr(member or "biomcp.exe", content)
    else:
        with tarfile.open(path, "w:gz") as archive:
            entry = tarfile.TarInfo(member or "biomcp")
            entry.size, entry.mode = len(content), 0o755
            archive.addfile(entry, io.BytesIO(content))
    checksum = hashlib.sha256(path.read_bytes()).hexdigest()
    path.with_name(name + ".sha256").write_text(checksum + "  " + name + "\n")
    return path, checksum


class Installer(unittest.TestCase):
    def test_download_gate_keeps_shell_tools_and_isolates_parent_configuration(self):
        with patch.object(sys, "path", [str(SCRIPT.parent), *sys.path]):
            from gate import installer_environment
        bash = shutil.which("bash") or "/bin/bash"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            marker = "synthetic-parent-marker"
            with patch.dict(os.environ, {key: marker for key in ["HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "HTTPS_PROXY", "HTTP_PROXY", "S2_API_KEY", "BASH_ENV"]}):
                env = installer_environment(root, bash)
            for name, leaf in [("HOME", "home"), ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache")]:
                self.assertEqual(env[name], str(root / leaf))
            self.assertNotIn(marker, env.values())
            if os.name == "nt":
                self.assertEqual(env["SystemRoot"], os.environ["SystemRoot"])
                self.assertNotIn(".", env["PATH"].split(os.pathsep))
            else:
                self.assertEqual(env["PATH"], os.defpath)
            commands = "for tool in mktemp uname unzip cp chmod mv rm tr; do command -v $tool || exit 1; done; command -v curl || command -v wget; command -v sha256sum || command -v shasum || command -v openssl"
            result = subprocess.run([bash, "-ec", commands], cwd=root, env=env, capture_output=True, text=True, timeout=20)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.destination = self.root / "installed"
        self.home = self.root / "home"
        self.home.mkdir()
        self.environment = {"PATH": str(self.bin) + os.pathsep + os.defpath, "HOME": str(self.home), "BIOMCP_INSTALL_DIR": str(self.destination)}

    def platform(self, system="Linux", machine="x86_64"):
        uname = self.bin / "uname"
        uname.write_text("#!/bin/sh\ncase \"$1\" in -s) echo '" + system + "';; -m) echo '" + machine + "';; esac\n")
        uname.chmod(0o755)

    def install(self, *args):
        return subprocess.run(["/bin/bash", str(SCRIPT), *args], env=self.environment, cwd=self.home, capture_output=True, text=True, timeout=20)

    def test_help_explains_provenance_and_checksum_before_installation(self):
        result = self.install("--help")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("gh attestation verify ARTIFACT --repo genomoncology/biomcp", result.stdout)
        self.assertIn("--signer-workflow genomoncology/biomcp/.github/workflows/release.yml", result.stdout)
        self.assertIn("--source-digest COMMIT", result.stdout)
        self.assertIn("--source-ref refs/tags/vX.Y.Z", result.stdout)
        self.assertIn("SHA-256 checksum", result.stdout)
        self.assertFalse(self.destination.exists())

    def test_empty_or_unset_home_needs_an_explicit_install_directory(self):
        archive, checksum = candidate(self.root, 'local.tar.gz')
        for home in (None, ''):
            with self.subTest(home=home):
                self.environment.pop('HOME', None)
                self.environment.pop('BIOMCP_INSTALL_DIR', None)
                if home is not None:
                    self.environment['HOME'] = home
                self.assertEqual(self.install('--help').returncode, 0)
                result = self.install()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('HOME is unset or empty', result.stderr)
                result = self.install(str(archive), checksum, str(self.destination))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.destination.joinpath('biomcp').unlink()

    def test_public_platform_archives_and_version_interface(self):
        platforms = [("Linux", "x86_64", "linux-x86_64", ".tar.gz"), ("Linux", "aarch64", "linux-arm64", ".tar.gz"), ("Darwin", "arm64", "darwin-arm64", ".tar.gz"), ("Darwin", "x86_64", "darwin-x86_64", ".tar.gz"), ("MSYS_NT-10.0", "x86_64", "windows-x86_64", ".zip"), ("MINGW64_NT-10.0", "amd64", "windows-x86_64", ".zip"), ("CYGWIN_NT-10.0", "x86_64", "windows-x86_64", ".zip")]
        for system, machine, platform, suffix in platforms:
            with self.subTest(system=system, machine=machine):
                self.platform(system, machine)
                release = self.root / "releases/download/v1.0.0"
                release.mkdir(parents=True, exist_ok=True)
                payload = self.root / "candidate"
                payload.write_bytes(b"#!/bin/sh\necho candidate\n")
                payload.chmod(0o755)
                if platform.startswith("windows"):
                    tag = "win_amd64"
                elif platform.startswith("darwin"):
                    tag = "macosx_11_0_" + platform.removeprefix("darwin-")
                else:
                    tag = "manylinux_2_28_" + ("aarch64" if platform.endswith("arm64") else "x86_64")
                write_payloads(release)
                archive = binary_archive(release, payload, tag)
                self.assertEqual(archive.name, "biomcp-" + platform + suffix)
                with https_server(self.root) as (base, ca):
                    self.environment["CURL_CA_BUNDLE"] = str(ca)
                    self.environment["BIOMCP_RELEASE_BASE_URL"] = base + "/releases"
                    result = self.install("--version", "1.0.0")
                self.assertEqual(result.returncode, 0, result.stderr)
                binary = self.destination / ("biomcp.exe" if suffix == ".zip" else "biomcp")
                self.assertEqual(binary.read_bytes(), b"#!/bin/sh\necho candidate\n")
                self.assertTrue(os.access(binary, os.X_OK))
                binary.unlink()

    def test_fixture_ca_reaches_curl_when_backend_ignores_bundle_environment(self):
        curl = shutil.which("curl", path=os.defpath)
        if not curl:
            self.skipTest("curl is required for the fixture CA forwarding proof")
        # Schannel ignores CURL_CA_BUNDLE; remove it before the real downloader
        # so this outside-in proof requires an explicit certificate argument.
        wrapper = self.bin / "curl"
        wrapper.write_text("#!/bin/sh\nunset CURL_CA_BUNDLE\nexec " + shlex.quote(curl) + " \"$@\"\n")
        wrapper.chmod(0o755)
        self.platform()
        release = self.root / "download/v1.0.0"
        release.mkdir(parents=True)
        candidate(release, "biomcp-linux-x86_64.tar.gz")
        with https_server(self.root) as (base, ca), https_server(self.root) as (_, other_ca):
            named_ca = self.root / "fixture CA.pem"
            shutil.copyfile(ca, named_ca)
            self.environment.update(BIOMCP_RELEASE_BASE_URL=base, BIOMCP_VERSION="1.0.0")
            for selected, passes in [(None, False), (other_ca, False), (named_ca, True)]:
                with self.subTest(ca=selected):
                    self.environment.pop("CURL_CA_BUNDLE", None)
                    if selected:
                        self.environment["CURL_CA_BUNDLE"] = str(selected)
                    result = self.install()
                    self.assertEqual(result.returncode == 0, passes, result.stderr)
                    self.assertEqual((self.destination / "biomcp").exists(), passes)
                    if passes:
                        self.assertEqual((self.destination / "biomcp").read_bytes(), b"#!/bin/sh\necho candidate\n")

    def test_latest_environment_and_local_tar_share_checksum_boundary(self):
        self.platform()
        release = self.root / "releases/latest/download"
        release.mkdir(parents=True)
        archive, checksum = candidate(release, "biomcp-linux-x86_64.tar.gz")
        with https_server(self.root) as (base, ca):
            self.environment["CURL_CA_BUNDLE"] = str(ca)
            self.environment["BIOMCP_RELEASE_BASE_URL"] = base + "/releases"
            self.assertEqual(self.install().returncode, 0)
        self.assertEqual(self.install(str(archive), checksum, str(self.root / "offline")).returncode, 0)
        unnamed = self.root / "local-archive"
        unnamed.write_bytes(archive.read_bytes())
        self.assertEqual(self.install(str(unnamed), checksum, str(self.root / "unnamed")).returncode, 0)
        self.environment["BIOMCP_VERSION"] = "v1.0.0"
        archive.with_name(archive.name + ".sha256").write_text("0" * 64 + "\n")
        self.destination.joinpath("biomcp").unlink()
        self.assertNotEqual(self.install(str(archive), "0" * 64, str(self.destination)).returncode, 0)
        self.assertFalse(self.destination.joinpath("biomcp").exists())

    def test_unverified_or_unsafe_download_never_installs(self):
        self.platform()
        release = self.root / "releases/download/v1.0.0"
        release.mkdir(parents=True)
        archive, checksum = candidate(release, "biomcp-linux-x86_64.tar.gz")
        with https_server(self.root) as (base, ca):
            self.environment["CURL_CA_BUNDLE"] = str(ca)
            self.environment.update(BIOMCP_RELEASE_BASE_URL=base + "/releases", BIOMCP_VERSION="v1.0.0")
            for bad in ["0" * 64, checksum + "  other.tar.gz", checksum + "\n" + checksum]:
                archive.with_name(archive.name + ".sha256").write_text(bad + "\n")
                result = self.install()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("checksum", result.stderr.lower())
                self.assertFalse(self.destination.exists())
            archive.with_name(archive.name + ".sha256").unlink()
            self.assertNotEqual(self.install().returncode, 0)
            for member in ["../escape", "bin/biomcp"]:
                candidate(release, archive.name, member=member)
                self.assertNotEqual(self.install().returncode, 0)
                self.assertFalse(self.destination.exists())
                self.assertFalse(self.root.joinpath("escape").exists())

    def test_zip_rejects_extra_paths_and_preserves_existing_install(self):
        self.platform("MSYS_NT-10.0")
        release = self.root / "releases/download/v1.0.0"
        release.mkdir(parents=True)
        self.destination.mkdir()
        installed = self.destination / "biomcp.exe"
        installed.write_bytes(b"previous executable")
        with https_server(self.root) as (base, ca):
            self.environment["CURL_CA_BUNDLE"] = str(ca)
            self.environment.update(BIOMCP_RELEASE_BASE_URL=base + "/releases", BIOMCP_VERSION="v1.0.0")
            archive, _ = candidate(release, "biomcp-windows-x86_64.zip", member="../escape.exe")
            result = self.install()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("archive contents", result.stderr)
            self.assertEqual(installed.read_bytes(), b"previous executable")
            self.assertFalse(self.root.joinpath("escape.exe").exists())
            archive.write_bytes(b"truncated archive")
            archive.with_name(archive.name + ".sha256").write_text(hashlib.sha256(archive.read_bytes()).hexdigest())
            self.assertNotEqual(self.install().returncode, 0)
            self.assertEqual(installed.read_bytes(), b"previous executable")

    def test_local_install_refuses_directory_and_symbolic_link_targets(self):
        archive, checksum = candidate(self.root, "local.tar.gz")
        self.destination.mkdir()
        target = self.destination / "biomcp"
        target.mkdir()
        result = self.install(str(archive), checksum, str(self.destination))
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(list(target.iterdir()), [])
        target.rmdir()
        other = self.root / "existing"
        other.write_bytes(b"keep")
        target.symlink_to(other)
        self.assertNotEqual(self.install(str(archive), checksum, str(self.destination)).returncode, 0)
        self.assertEqual(other.read_bytes(), b"keep")
        target.unlink()
        directory_link = self.root / "directory-link"
        directory_link.symlink_to(self.destination, target_is_directory=True)
        for suffix in ["", "/", "/.", "/./"]:
            result = self.install(str(archive), checksum, str(directory_link) + suffix)
            self.assertNotEqual(result.returncode, 0, suffix)
            self.assertIn("symbolic-link install directory", result.stderr)
            self.assertFalse(target.exists())

    def test_downloaders_follow_https_redirects_and_never_contact_http_targets(self):
        class Counter(QuietHandler):
            requests = 0
            def do_GET(self):
                Counter.requests += 1
                super().do_GET()
        class Redirect(QuietHandler):
            destination = ""
            def do_GET(self):
                if self.path.startswith("/safe/"):
                    self.path = self.path.removeprefix("/safe")
                    return super().do_GET()
                self.send_response(302)
                self.send_header("Location", Redirect.destination + self.path)
                self.end_headers()
        self.platform()
        release = self.root / "download/v1.0.0"
        release.mkdir(parents=True)
        candidate(release, "biomcp-linux-x86_64.tar.gz")
        for tool in ["curl", "wget"]:
            executable = shutil.which(tool)
            if not executable:
                print("Installer redirect proof unavailable: " + tool)
                continue
            print("Installer redirect proof uses real " + tool)
            with self.subTest(downloader=tool), server(self.root, handler=Counter) as insecure, https_server(self.root, handler=Redirect) as (base, ca):
                for name in ["curl", "wget"]:
                    (self.bin / name).unlink(missing_ok=True)
                (self.bin / tool).symlink_to(executable)
                for name in ["mktemp", "mkdir", "rm", "tr", "tar", "gzip", "cp", "chmod", "mv", "shasum", "sha256sum", "openssl"]:
                    source = shutil.which(name, path=os.defpath)
                    if source and not (self.bin / name).exists():
                        (self.bin / name).symlink_to(source)
                self.environment.update(PATH=str(self.bin), BIOMCP_RELEASE_BASE_URL=base,
                                        BIOMCP_VERSION="1.0.0", CURL_CA_BUNDLE=str(ca))
                config = self.root / "wgetrc"
                config.write_text("ca_certificate=" + str(ca) + "\n")
                self.environment["WGETRC"] = str(config)
                for destination in [base + "/safe", "/safe", "//" + base.removeprefix("https://") + "/safe", "../../../safe"]:
                    Redirect.destination = destination
                    result = self.install()
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.destination.joinpath("biomcp").unlink()
                Redirect.destination = insecure
                result = self.install()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(Counter.requests, 0)
                self.assertFalse(self.destination.joinpath("biomcp").exists())

    def test_plain_http_base_is_refused_before_download(self):
        self.platform()
        self.environment["BIOMCP_RELEASE_BASE_URL"] = "http://127.0.0.1:1/releases"
        result = self.install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("HTTPS", result.stderr)
        self.assertFalse(self.destination.exists())

    def test_unknown_platform_names_failure_and_help_needs_no_download(self):
        for system, machine, message in [("Plan9", "x86_64", "Plan9"), ("Linux", "mips", "mips"), ("MSYS_NT", "arm64", "windows/arm64")]:
            self.platform(system, machine)
            result = self.install()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(message, result.stderr)
            self.assertFalse(self.destination.exists())
        self.assertEqual(self.install("--help").returncode, 0)


if __name__ == "__main__":
    unittest.main()
