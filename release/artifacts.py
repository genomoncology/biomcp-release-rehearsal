#!/usr/bin/env python3
"""Build and inspect local binary artifacts. Never publish or fetch a release."""
import argparse
import base64
import csv
from email.parser import Parser
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import sys
import subprocess
import tarfile
import zipfile

if __package__:
    from .vendor import environment as vendor_environment, stage as vendor_stage
    from .readme import description
    from .tools import candidate, check
    from .source_revision import revision as source_revision
    from .notices import generate, payloads, metadata as check_metadata
    from .license_check import zip_payload
else:
    from vendor import environment as vendor_environment, stage as vendor_stage
    from readme import description
    from tools import candidate, check
    from source_revision import revision as source_revision
    from notices import generate, payloads, metadata as check_metadata
    from license_check import zip_payload


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(args):
    return subprocess.check_output(args, text=True,
                                   env=vendor_environment() if args[:1] == ["cargo"] else None)


def cargo(profile, tests=False, revision=None, clean_paths=False):
    args = ["cargo", "test" if tests else "build", "--locked", "-p", "biomcp", "--profile", profile, "--message-format=json"]
    target = tests if isinstance(tests, str) else "cases"
    args += ["--test", target, "--no-run"] if tests else ["--bin", "biomcp"]
    environment = {**vendor_environment(clean_paths), **({"BIOMCP_BUILD_REVISION": revision} if revision else {})}
    process = subprocess.Popen(args, stdout=subprocess.PIPE, text=True, env=environment)
    found = None
    for line in process.stdout:
        message = json.loads(line)
        if message.get("reason") == "compiler-message" and message["message"].get("rendered"):
            print(message["message"]["rendered"], file=sys.stderr, end="")
        if message.get("reason") == "compiler-artifact" and message.get("executable") and message["target"]["name"] == (target if tests else "biomcp"):
            found = message
    if process.wait() or not found:
        raise ValueError("Cargo did not produce the requested executable")
    return found


def platform_tag(manylinux):
    arch = {"aarch64": "arm64", "AMD64": "x86_64"}.get(platform.machine(), platform.machine())
    if platform.system() == "Darwin":
        # Binary deployment floor comes from the producer's MACOSX_DEPLOYMENT_TARGET.
        return "macosx_11_0_" + arch
    if platform.system() == "Linux":
        return ("manylinux_2_28_" if manylinux else "linux_") + {"arm64": "aarch64"}.get(arch, arch)
    if platform.system() == "Windows":
        return "win_amd64"
    raise ValueError("unsupported build platform")


def check_floor(binary, tag):
    if not tag.startswith("manylinux_"):
        return
    info = subprocess.check_output(["readelf", "--version-info", "--wide", str(binary)], text=True)
    required = [tuple(map(int, value.split("."))) for value in re.findall(r"\bGLIBC_([0-9.]+)", info)]
    floor = tuple(map(int, tag.split("_")[1:3]))
    if not required or max(required) > floor:
        raise ValueError("ELF requirements exceed the claimed manylinux floor")


def check_build_paths(binary):
    """Refuse a binary that names a folder of the machine that built it."""
    # In a container the root is /source and the home may be /tmp, so a search
    # for either bare folder would match ordinary text.
    home = Path(os.environ.get("CARGO_HOME") or Path.home() / ".cargo")
    folders = [Path(__file__).resolve().parent.parent / "crates", binary.resolve().parent, vendor_stage(), home / "registry"]
    data = binary.read_bytes()
    found = {str(folder) for folder in folders for separator in "/\\"
             if str(folder).replace("\\", "/").replace("/", separator).encode() in data}
    if found:
        raise ValueError("Release binary holds build paths: " + ", ".join(sorted(found)))


def wheel(output, binary, provenance):
    version, tag = provenance["version"], provenance["platform"]
    stem = provenance["distribution"].replace("-", "_") + "-" + version
    metadata = stem + ".dist-info"
    suffix = ".exe" if tag.startswith("win_") else ""
    files = {
        stem + ".data/scripts/biomcp" + suffix: binary.read_bytes(),
        stem + ".data/scripts/" + provenance["distribution"] + suffix: binary.read_bytes(),
        metadata + "/METADATA": ("Metadata-Version: 2.1\nName: " + provenance["distribution"] + "\nVersion: " + version + "\nSummary: BioMCP CLI and MCP server\nRequires-Python: >=3.9\nLicense: MIT\nProject-URL: Homepage, https://biomcp.org\nProject-URL: Repository, https://github.com/genomoncology/biomcp\nProject-URL: Documentation, https://biomcp.org\nDescription-Content-Type: text/markdown\n\n" + description() + "\n\nmcp-name: io.github.genomoncology/biomcp\n").encode(),
        metadata + "/WHEEL": ("Wheel-Version: 1.0\nGenerator: biomcp-local-producer\nRoot-Is-Purelib: false\nTag: py3-none-" + tag + "\n").encode(),
        metadata + "/release.json": json.dumps(provenance, sort_keys=True).encode(),
    }
    files.update({metadata + "/" + name: data for name, data in payloads(output).items()})
    record = io.StringIO(newline="")
    writer = csv.writer(record)
    for name, data in files.items():
        checksum = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
        writer.writerow([name, "sha256=" + checksum, len(data)])
    writer.writerow([metadata + "/RECORD", "", ""])
    files[metadata + "/RECORD"] = record.getvalue().encode()
    path = output / (stem + "-py3-none-" + tag + ".whl")
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            entry = zipfile.ZipInfo(name)
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = (0o100755 if ".data/scripts/" in name else 0o100644) << 16
            archive.writestr(entry, data)
    return path


def inspect(path):
    with zipfile.ZipFile(path) as archive:
        manifests = [n for n in archive.namelist() if n.endswith(".dist-info/release.json")]
        binaries = [n for n in archive.namelist() if ".data/scripts/" in n]
        if len(manifests) != 1:
            raise ValueError("wheel must contain one producer record")
        record = json.loads(archive.read(manifests[0]))
        metadata = Parser().parsestr(archive.read(manifests[0].replace("release.json", "METADATA")).decode())
        check_metadata(metadata)
        zip_payload(path, manifests[0].removesuffix("release.json"))
        if metadata["Name"] != record["distribution"] or metadata["Version"] != record["version"]:
            raise ValueError("wheel package identity differs from its producer")
        expected_tag = "py3-none-" + record["platform"]
        if not path.name.endswith("-" + expected_tag + ".whl") or ("Tag: " + expected_tag + "\n").encode() not in archive.read(manifests[0].replace("release.json", "WHEEL")):
            raise ValueError("wheel tags differ from the producer platform")
        profile = record["cargo_profile"]
        if record["profile"] != "release" or profile["debug_assertions"] or profile["opt_level"] != "3":
            raise ValueError("release profile required")
        suffix = ".exe" if record["platform"].startswith("win_") else ""
        prefix = manifests[0].split(".dist-info/")[0] + ".data/scripts/"
        if set(binaries) != {prefix + "biomcp" + suffix, prefix + record["distribution"] + suffix}:
            raise ValueError("wheel executable entry points differ")
        if any(digest(archive.read(name)) != record["binary_sha256"] for name in binaries):
            raise ValueError("wheel binary differs from Cargo's candidate bytes")
        return record


def candidate_record(artifacts, revision, version=None):
    """Bind requested source identity to the same inspected wheel the gate uses."""
    wheels = list(artifacts.glob('*.whl'))
    if len(wheels) != 1:
        raise ValueError('candidate must contain exactly one wheel')
    record = inspect(wheels[0])
    if record != json.loads((artifacts / 'release.json').read_text()):
        raise ValueError('candidate sidecar differs from inspected wheel')
    if record['revision'] != revision or (version is not None and record['version'] != version):
        raise ValueError('candidate source revision or version differs')
    return record


def channels(output, binary, provenance):
    source = Path(__file__).resolve().parent.parent
    check(source)
    manifest = json.loads((source / "manifest.json").read_text())
    manifest["tools"] = candidate(binary)
    if sorted(tool["name"] for tool in manifest["tools"]) != sorted(tool["name"] for tool in json.loads((source / "manifest.json").read_text())["tools"]):
        raise ValueError("desktop manifest tools differ from candidate server")
    manifest["version"] = provenance["version"]
    manifest["compatibility"] = {"platforms": [{"Darwin": "darwin", "Windows": "win32", "Linux": "linux"}[platform.system()]]}
    entry = "server/biomcp" + (".exe" if platform.system() == "Windows" else "")
    manifest["server"]["entry_point"] = entry
    manifest["server"]["mcp_config"]["command"] = "${__dirname}/" + entry
    with zipfile.ZipFile(output / ("biomcp-" + provenance["platform"] + ".mcpb"), "w", zipfile.ZIP_DEFLATED) as bundle:
        for name, data in payloads(output).items():
            bundle.writestr(name, data)
        bundle.writestr("manifest.json", json.dumps(manifest, sort_keys=True))
        member = zipfile.ZipInfo(entry)
        member.external_attr = 0o100755 << 16
        bundle.writestr(member, binary.read_bytes())
    registry = json.loads((source / "server.json").read_text())
    registry["version"] = provenance["version"]
    for package in registry["packages"]:
        package["version"] = provenance["version"]
    (output / "server.json").write_text(json.dumps(registry, indent=2) + "\n")


def binary_archive(output, binary, tag):
    """Write the public installer's platform asset from the candidate bytes."""
    if tag.startswith("win_"):
        name = "biomcp-windows-x86_64.zip"
    elif tag.startswith("macosx_"):
        name = "biomcp-darwin-" + ("arm64" if tag.endswith("arm64") else "x86_64") + ".tar.gz"
    else:
        name = "biomcp-linux-" + ("arm64" if tag.endswith("aarch64") else "x86_64") + ".tar.gz"
    archive = output / name
    if name.endswith(".zip"):
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as package:
            entry = zipfile.ZipInfo("biomcp.exe")
            entry.external_attr = 0o100755 << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            package.writestr(entry, binary.read_bytes())
            for member, data in payloads(output).items():
                package.writestr(member, data)
    else:
        with tarfile.open(archive, "w:gz") as package:
            package.add(binary, arcname="biomcp", recursive=False)
            for member, data in payloads(output).items():
                entry = tarfile.TarInfo(member)
                entry.size = len(data)
                entry.mode = 0o644
                package.addfile(entry, io.BytesIO(data))
    (output / (name + ".sha256")).write_text(digest(archive.read_bytes()) + "  " + name + "\n")
    return archive


def build(args):
    # Pin the macOS deployment target before Cargo sees the build environment.
    if platform.system() == "Darwin":
        os.environ["MACOSX_DEPLOYMENT_TARGET"] = "11.0"
    revision = source_revision(args.revision)
    artifact = cargo(args.profile, revision=revision, clean_paths=args.profile == "release")
    binary = Path(artifact["executable"])
    metadata = json.loads(run(["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"]))
    version = next(p["version"] for p in metadata["packages"] if p["name"] == "biomcp")
    tag = platform_tag(args.manylinux)
    check_floor(binary, tag)
    if args.profile == "release":
        check_build_paths(binary)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    distribution = json.loads((Path(__file__).resolve().parent.parent / "server.json").read_text())["packages"][0]["identifier"]
    record = {"distribution": distribution, "version": version, "platform": tag, "profile": args.profile, "cargo_profile": artifact["profile"], "binary_sha256": digest(binary.read_bytes()), "revision": revision}
    generate(output)
    candidate = wheel(output, binary, record)
    channels(output, binary, record)
    archive = binary_archive(output, binary, tag)
    (output / "release.json").write_text(json.dumps(record, indent=2) + "\n")
    formula = '''class Biomcp < Formula
  desc "BioMCP CLI and MCP server"
  homepage "https://biomcp.org"
  version "VERSION"
  url "URL"
  sha256 "SHA"
  def install
    bin.install "biomcp"
  end
end
'''.replace("VERSION", version).replace("URL", archive.as_uri()).replace("SHA", digest(archive.read_bytes()))
    (output / "biomcp.rb").write_text(formula)
    if args.profile == "release":
        inspect(candidate)
    print(json.dumps({"wheel": str(candidate), "archive": str(archive), "profile": args.profile}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    producer = commands.add_parser("build")
    producer.add_argument("--manylinux", action="store_true", help="Inspect ELF requirements before claiming the manylinux floor")
    producer.add_argument("--revision", help="Source revision when building a git archive")
    producer.add_argument("--output", type=Path, required=True)
    producer.add_argument("--profile", choices=["release", "dev"], default="release")
    inspector = commands.add_parser("inspect")
    inspector.add_argument("wheel", type=Path)
    args = parser.parse_args()
    if args.command == "build":
        build(args)
    else:
        print(json.dumps(inspect(args.wheel), sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error))
