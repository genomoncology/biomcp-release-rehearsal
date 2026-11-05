#!/usr/bin/env python3
"""Install local candidates and exercise them outside the checkout. No publication."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import zipfile

if __package__:
    from .artifacts import cargo, inspect
    from .source_revision import revision as source_revision
    from .notices import NAMES as LEGAL_NAMES
    from .windows import check_runtime
    from .smoke import environment, smoke
    from .local_server import https_server
else:
    from artifacts import cargo, inspect
    from source_revision import revision as source_revision
    from notices import NAMES as LEGAL_NAMES
    from windows import check_runtime
    from smoke import environment, smoke
    from local_server import https_server



def installer_environment(root, bash):
    env = environment(root)
    path = os.defpath
    if os.name == "nt":
        shell = Path(bash).resolve(strict=True).parent
        prefix = shell.parent.parent if shell.parent.name.lower() == "usr" else shell.parent
        # Git Bash keeps POSIX utilities and the downloader in these directories.
        directories = [shell, prefix / "usr/bin", prefix / "mingw64/bin"]
        if "SystemRoot" in env:
            directories.append(Path(env["SystemRoot"]) / "System32")
        path = os.pathsep.join(dict.fromkeys(str(item) for item in directories if item.is_dir()))
    return {**env, "PATH": path}


def install_wheel(wheel, venv, root, host_python, name="biomcp"):
    subprocess.run([host_python, "-m", "venv", "--without-pip", str(venv)], check=True)
    python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    install_env = {**environment(root), "PATH": os.defpath, "PIP_CONFIG_FILE": os.devnull, "PIP_NO_CACHE_DIR": "1", "UV_CACHE_DIR": str(root / "uv-cache")}
    uv = shutil.which("uv")
    if uv:
        installer = [uv, "--offline", "--no-config", "pip", "install", "--python", str(python)]
    else:
        subprocess.run([str(python), "-m", "ensurepip"], check=True, env=install_env)
        installer = [str(python), "-m", "pip", "install"]
    subprocess.run(installer + ["--no-index", "--no-deps", str(wheel)], check=True, env=install_env)
    return venv / (("Scripts/" + name + ".exe") if os.name == "nt" else ("bin/" + name))


def cli_checks(binary, record):
    """Run literal CLI commands with every source group disabled and no host keys."""
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory).resolve()
        env = environment(root)
        config = root / "config/biomcp/config.json"
        config.parent.mkdir(parents=True)
        groups = ["open", "noncommercial", "restricted"]
        config.write_text(json.dumps({"disable_sources": groups}))
        # Terms groups preserve aggregators with independently labeled components.
        # Discover their IDs offline from the installed candidate, then disable
        # every source explicitly before the recorded checks.
        discovered = subprocess.run([str(binary.resolve()), "health", "--offline"], env=env, cwd=root,
                                    capture_output=True, text=True, timeout=60)
        if discovered.returncode or discovered.stderr:
            raise ValueError("installed CLI command failed: " + discovered.stdout + discovered.stderr)
        discovery = json.loads(discovered.stdout)
        names = [row["source"] for row in discovery["result"]["sources"]]
        if discovery["status"] != "complete" or not names or any(not isinstance(name, str) or not name for name in names):
            raise ValueError("installed CLI source discovery failed")
        config.write_text(json.dumps({"disable_sources": groups + names}))
        rows = []
        for command in [["version"], ["version", "--verbose"], ["health", "--offline"]]:
            result = subprocess.run([str(binary.resolve())] + command, env=env, cwd=root,
                                    capture_output=True, text=True, timeout=60)
            if result.returncode or result.stderr:
                raise ValueError("installed CLI command failed: " + result.stdout + result.stderr)
            if command == ["version"]:
                output = result.stdout
                if output != "biomcp " + record["version"] + "\n":
                    raise ValueError("installed CLI version differs")
            elif command == ["version", "--verbose"]:
                output = result.stdout
                if not output.startswith('biomcp ' + record['version'] + '\n') or output.splitlines().count('Commit: ' + record['revision']) != 1:
                    raise ValueError('installed CLI commit differs from candidate')
            else:
                output = json.loads(result.stdout)
                sources = output["result"]["sources"]
                if output["status"] != "complete" or not sources or any(
                    row.get("disabled") is not True or row.get("state") != "skipped"
                    or row.get("reason") != "disabled" or "latency_ms" in row for row in sources
                ):
                    raise ValueError("installed CLI health did not skip every source")
            rows.append({"command": command, "output": output, "result": "pass",
                         "binary_sha256": record["binary_sha256"], "revision": record["revision"]})
        return rows


def stage_fixtures(source, fixtures):
    shutil.copytree(source / "tests", fixtures, dirs_exist_ok=True)
    provenance = Path("sdlc/planning/slices/gene.md")
    (fixtures / "support" / provenance).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source / provenance, fixtures / "support" / provenance)
    helpers = fixtures / "support/scripts"
    helpers.mkdir(parents=True, exist_ok=True)
    for name in ("icite_discovery_proof.py", "nightly_live_check.py"):
        shutil.copy2(source / "scripts" / name, helpers / name)


def installed(args):
    output = args.artifacts.resolve()
    wheel = next(output.glob("*.whl"))
    record = inspect(wheel)  # All distributions must match these Cargo candidate bytes.
    if not re.fullmatch(r"[0-9a-f]{40}", record["revision"]):
        raise ValueError("candidate revision must be a full Git commit")
    if record["platform"].startswith("win_"):
        with zipfile.ZipFile(wheel) as package:
            entry, = [name for name in package.namelist() if name.endswith(".data/scripts/biomcp.exe")]
            check_runtime(package.read(entry), record["platform"])
    root = args.scratch.resolve()
    root.mkdir(parents=True, exist_ok=True)
    fixtures = root / "fixtures"
    source = Path(__file__).resolve().parent.parent
    executable = root / ("case-runner.exe" if os.name == "nt" else "case-runner")
    if not args.prepared:
        tooling_revision = source_revision()
        stage_fixtures(source, fixtures)
        shutil.copy2(Path(cargo("release", tests=True, revision=tooling_revision)["executable"]), executable)
        tool = executable.with_name("tool-runner" + executable.suffix)
        shutil.copy2(Path(cargo("release", tests="tools", revision=tooling_revision)["executable"]), tool)
        shutil.copytree(source / "release", root / "tools/release", dirs_exist_ok=True, ignore=shutil.ignore_patterns("__pycache__"))
        shutil.copy2(source / "LICENSE", root / "tools/LICENSE")
        bootstrap = {"tooling_revision": tooling_revision, "product_revision": record["revision"],
                     "python": args.python}
        (root / "bootstrap.json").write_text(json.dumps(bootstrap, sort_keys=True))
        print(json.dumps({"bootstrap": bootstrap}), flush=True)
    if args.prepare_only:
        return
    results = []
    suite_timeout = None
    def check(kind, binary, server_args=("serve",), declared_tools=None):
        nonlocal suite_timeout
        if hashlib.sha256(binary.read_bytes()).hexdigest() != record["binary_sha256"]:
            raise ValueError(kind + " installed bytes differ from the candidate")
        check_runtime(binary.read_bytes(), record["platform"])
        for row in cli_checks(binary, record):
            results.append({"artifact": kind, "platform": record["platform"], **row})
            print(json.dumps(results[-1]), flush=True)
        completed = smoke(binary, executable, fixtures, record["version"], server_args, suite_timeout,
                          python=args.python)
        if suite_timeout is None:
            suite_timeout = completed["timeout_seconds"]
        if declared_tools is not None and sorted(completed["tools"]) != sorted(declared_tools):
            raise ValueError("desktop manifest tools differ from installed server")
        results.append({"artifact": kind, "platform": record["platform"], "result": "pass", **{key: value for key, value in completed.items() if key not in ("tools", "checks")}, "checks": ["candidate-bytes"] + (["windows-static-crt"] if record["platform"].startswith("win_") else []) + completed["checks"]})
        print(json.dumps(results[-1]), flush=True)
    check("wheel", install_wheel(wheel, root / "venv", root, args.python))
    bundle = next(output.glob("*.mcpb"))
    extension = root / "desktop-extension"
    with zipfile.ZipFile(bundle) as package:
        manifest = json.loads(package.read("manifest.json"))
        entry = manifest["server"]["entry_point"]
        if set(package.namelist()) != {"manifest.json", entry, *LEGAL_NAMES} or entry not in {"server/biomcp", "server/biomcp.exe"}:
            raise ValueError("unexpected desktop bundle members")
        extension.mkdir()
        binary = extension / entry
        binary.parent.mkdir()
        binary.write_bytes(package.read(entry))
        binary.chmod(0o755)
        (extension / "manifest.json").write_bytes(package.read("manifest.json"))
    config = manifest["server"]["mcp_config"]
    if manifest["name"] != "biomcp" or manifest["version"] != record["version"] or config["command"] != "${__dirname}/" + entry or config["args"] != ["serve"]:
        raise ValueError("desktop manifest launch contract differs")
    check("desktop-extension", binary, config["args"], [tool["name"] for tool in manifest["tools"]])
    registry = json.loads((output / "server.json").read_text())
    package, = registry["packages"]
    if registry["name"] != "io.github.genomoncology/biomcp" or registry["version"] != record["version"] or package["registryType"] != "pypi" or package["identifier"] != record["distribution"] or package["version"] != record["version"] or package["transport"] != {"type": "stdio"}:
        raise ValueError("registry identity, package or transport differs")
    arguments = [argument["value"] for argument in package["packageArguments"] if argument["type"] == "positional"]
    if arguments != ["serve"]:
        raise ValueError("registry package launch contract differs")
    registry_binary = install_wheel(wheel, root / "registry-venv", root, args.python, package["identifier"])
    check("mcp-registry", registry_binary, arguments)
    archive = next(path for path in output.iterdir() if path.name.endswith((".tar.gz", ".zip")))
    suffix = ".exe" if archive.name.endswith(".zip") else ""
    bash = shutil.which("bash") or "/bin/bash"
    destination = root / "installer-bin"
    subprocess.run([bash, (source / "release/install.sh").as_posix(), archive.as_posix(), hashlib.sha256(archive.read_bytes()).hexdigest(), destination.as_posix()], check=True)
    check("install-script", destination / ("biomcp" + suffix))
    downloads = root / "downloads"
    release = downloads / "download" / ("v" + record["version"])
    release.mkdir(parents=True)
    for item in [archive, archive.with_name(archive.name + ".sha256")]:
        shutil.copy2(item, release / item.name)
    public_destination = root / "download-installer-bin"
    with https_server(downloads, shutil.which("openssl", path=installer_environment(root, bash)["PATH"]) or "openssl") as (base, ca):
        download_env = {**installer_environment(root, bash), "BIOMCP_RELEASE_BASE_URL": base, "BIOMCP_INSTALL_DIR": public_destination.as_posix(), "CURL_CA_BUNDLE": ca.as_posix()}
        # Identify the actual downloader and CA readability in this shell's
        # isolated context, without exposing parent configuration or credentials.
        diagnostic = (
            'if command -v curl >/dev/null 2>&1; then '
            'printf "Installer downloader: %s\\n" "$(command -v curl)"; curl --version; fi; '
            'test -r "$CURL_CA_BUNDLE"; '
            'printf "Installer fixture CA readable: %s\\n" "$CURL_CA_BUNDLE"'
        )
        subprocess.run([bash, "-ec", diagnostic], env=download_env, check=True)
        subprocess.run([bash, (source / "release/install.sh").as_posix(), "--version", record["version"]], env=download_env, check=True)
    check("download-install-script", public_destination / ("biomcp" + suffix))
    if not homebrew(output, root, check):
        results.append({"artifact": "homebrew-formula", "platform": record["platform"], "result": "deferred", "reason": "Homebrew is unavailable"})
    (args.report or root / "report.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results))


def staged_homebrew(args):
    """Install the publication formula using its exact checksums and local staged archives."""
    output = args.staged_formula.resolve().parent
    record = json.loads((args.artifacts / 'release.json').read_text())
    root = args.scratch.resolve()
    root.mkdir(parents=True, exist_ok=True)
    rows = []
    def check(kind, binary):
        if hashlib.sha256(binary.read_bytes()).hexdigest() != record['binary_sha256']:
            raise ValueError('staged Homebrew binary differs from candidate')
        rows.extend({'artifact': 'staged-homebrew-formula', 'platform': record['platform'], **row}
                    for row in cli_checks(binary, record))
    if args.staged_formula.name != 'biomcp.rb' or not homebrew(output, root, check, staged=True):
        raise ValueError('staged formula installation requires Homebrew')
    (args.report or root / 'report.json').write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps(rows), flush=True)


def homebrew(output, root, check, staged=False):
    """Install the candidate in an owned local tap and remove the whole prefix."""
    brew = shutil.which("brew")
    if not brew:
        return False
    brew_repo = subprocess.check_output([brew, "--repository"], text=True).strip()
    with tempfile.TemporaryDirectory(prefix="homebrew-", dir=root) as directory:
        owned = Path(directory)
        prefix = owned / "prefix"
        subprocess.run(["git", "clone", "--quiet", "--shared", "--local", brew_repo, str(prefix)], check=True)
        tap = prefix / "Library/Taps/biomcp/homebrew-candidate"
        formula = tap / "Formula/biomcp.rb"
        formula.parent.mkdir(parents=True)
        subprocess.run(["git", "init", "--quiet", str(tap)], check=True)
        text = (output / 'biomcp.rb').read_text()
        if staged:
            version = json.loads((output / 'candidate.json').read_text())['version']
            for arch in ('arm64', 'x86_64'):
                archive = output / ('biomcp-darwin-' + arch + '.tar.gz')
                url = 'https://github.com/genomoncology/biomcp/releases/download/v' + version + '/' + archive.name
                if url not in text or not archive.is_file():
                    raise ValueError('staged Homebrew formula must retain both candidate architectures')
                text = text.replace(url, archive.as_uri())
        formula.write_text(text)
        formula_env = {**environment(owned), "PATH": os.defpath + ":" + str(prefix / "bin"), "HOMEBREW_NO_AUTO_UPDATE": "1", "HOMEBREW_NO_INSTALL_FROM_API": "1", "HOMEBREW_NO_ANALYTICS": "1", "HOMEBREW_NO_ENV_HINTS": "1", "HOMEBREW_CACHE": str(owned / "cache"), "HOMEBREW_LOGS": str(owned / "logs"), "HOMEBREW_TEMP": str(owned / "tmp")}
        subprocess.run([str(prefix / "bin/brew"), "install", "--formula", "--ignore-dependencies", "biomcp/candidate/biomcp"], env=formula_env, check=True)
        check("homebrew-formula", prefix / "bin/biomcp")
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--report", type=Path, help="Write the complete JSON report here instead of scratch/report.json")
    parser.add_argument("--prepared", action="store_true", help="Use the already copied runner and public fixtures")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--python", default=sys.executable)
    parser.add_argument('--staged-formula', type=Path, help='Install the staged universal formula using local archives and verify candidate identity')
    args = parser.parse_args()
    if args.staged_formula:
        staged_homebrew(args)
    else:
        installed(args)


if __name__ == "__main__":
    main()
