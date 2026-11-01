"""Prepare one candidate's public release payloads without publishing."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import zipfile

if __package__:
    from .notes import schema_changes
    from .toolchain import manifest
else:
    from notes import schema_changes
    from toolchain import manifest


if __package__:
    from .license_check import native, zip_payload
    from .notices import payloads
else:
    from license_check import native, zip_payload
    from notices import payloads


PLATFORMS = [
    {"os": "ubuntu-24.04", "platform": "manylinux_2_28_x86_64", "arch": "amd64", "image": manifest()["images"]["manylinux-x86_64"]},
    {"os": "ubuntu-24.04-arm", "platform": "manylinux_2_28_aarch64", "arch": "arm64", "image": manifest()["images"]["manylinux-aarch64"]},
    {"os": "macos-15-intel", "platform": "macosx_11_0_x86_64", "arch": "", "image": ""},
    {"os": "macos-14", "platform": "macosx_11_0_arm64", "arch": "", "image": ""},
    {"os": "windows-2022", "platform": "win_amd64", "arch": "", "image": ""},
]


def identity(args):
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], text=True))
    version = next(package["version"] for package in metadata["packages"] if package["name"] == "biomcp")
    if not re.fullmatch(r"[0-9a-f]{40}", args.revision):
        raise ValueError("candidate revision must be a full Git commit")
    if args.event == "push" and args.tag != "v" + version:
        raise ValueError("release tag must match the committed Cargo version")
    print("version=" + version)
    print("matrix=" + json.dumps({"include": PLATFORMS}, separators=(",", ":")))


def candidates(args):
    records = {}
    for path in args.artifacts.glob("*/release.json"):
        record = json.loads(path.read_text())
        tag = record["platform"]
        if tag in records:
            raise ValueError("duplicate native platform")
        if record["version"] != args.version or record["revision"] != args.revision or record["distribution"] != "biomcp-cli":
            raise ValueError("staged artifacts must share the candidate identity")
        records[tag] = path.parent
    if set(records) != {row["platform"] for row in PLATFORMS}:
        raise ValueError("all five native release platforms are required")
    return records


def copy_native(directory, output, tag, registry):
    native(directory, tag)
    for pattern in ["*.whl", "*.tar.gz", "*.zip"]:
        for path in directory.glob(pattern):
            shutil.copy2(path, output / path.name)
    wheel, = directory.glob("*.whl")
    if not wheel.name.endswith("-py3-none-" + tag + ".whl"):
        raise ValueError("wheel platform differs from candidate record")
    archive, = [path for path in directory.iterdir() if path.name.endswith((".tar.gz", ".zip"))]
    checksum = archive.with_name(archive.name + ".sha256")
    expected = hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name
    if checksum.read_text().strip() != expected:
        raise ValueError("archive checksum differs from candidate bytes")
    shutil.copy2(checksum, output / checksum.name)
    if json.loads((directory / "server.json").read_text()) != registry:
        raise ValueError("registry payload differs between native producers")
    # Desktop support is promised on macOS and Windows only.
    if tag.startswith(("macosx_", "win_")):
        bundle, = directory.glob("*.mcpb")
        shutil.copy2(bundle, output / bundle.name)
    if tag.startswith("manylinux_"):
        arch = "amd64" if tag.endswith("x86_64") else "arm64"
        image = directory / ("biomcp-container-" + arch + ".tar")
        shutil.copy2(image, output / image.name)


def formula(output, version):
    lines = ['class Biomcp < Formula', '  desc "BioMCP CLI and MCP server"',
             '  homepage "https://biomcp.org"', '  version "' + version + '"',
             '  on_macos do']
    for arch, selector in [("arm64", "on_arm"), ("x86_64", "on_intel")]:
        name = "biomcp-darwin-" + arch + ".tar.gz"
        checksum = (output / (name + ".sha256")).read_text().split()[0]
        lines += ['    ' + selector + ' do',
                  '      url "https://github.com/genomoncology/biomcp/releases/download/v' + version + '/' + name + '"',
                  '      sha256 "' + checksum + '"', '    end']
    lines += ['  end', '  def install', '    bin.install "biomcp"', '  end', 'end', '']
    (output / "biomcp.rb").write_text("\n".join(lines))


def plugin(source, output, version, notices):
    metadata = json.loads((source / ".claude-plugin/plugin.json").read_text())
    marketplace = json.loads((source / ".claude-plugin/marketplace.json").read_text())
    entry, = marketplace["plugins"]
    if metadata["name"] != "biomcp" or marketplace["name"] != "biomcp" or entry["name"] != "biomcp" or entry["source"] != "./":
        raise ValueError("plugin and marketplace must preserve the candidate identity")
    if metadata.get("skills") != ["./skills/"]:
        raise ValueError("plugin must discover the canonical skills directory")
    if metadata.get('version') != version:
        raise ValueError('plugin manifest must carry the candidate version')
    if entry["mcpServers"] != {"biomcp": {"command": "biomcp", "args": ["serve"]}}:
        raise ValueError("plugin must launch installed biomcp serve")
    guide = source / "skills/SKILL.md"
    if not guide.is_file():
        raise ValueError("plugin must contain the release skill guide")
    metadata["version"] = entry["version"] = version
    paths = [path for path in (source / "skills").rglob("*") if path.is_file()]
    with zipfile.ZipFile(output / "biomcp-plugin.zip", "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in payloads(notices).items():
            archive.writestr(name, data)
        for name, value in [("plugin", metadata), ("marketplace", marketplace)]:
            archive.writestr(".claude-plugin/" + name + ".json", json.dumps(value, indent=2) + "\n")
        for path in sorted(paths):
            archive.write(path, path.relative_to(source).as_posix())


def release_notes(output, version, revision):
    (output / "RELEASE-NOTES.md").write_text(
        (Path(__file__).with_name("notes.md").read_text() + f"\n\nCandidate source commit: `{revision}`.\n\n"
         + schema_changes() + "\n\n")
        + "Before installation, verify each downloaded release artifact with a current GitHub CLI:\n\n"
        "```sh\ngh attestation verify ARTIFACT --repo genomoncology/biomcp "
        "--signer-workflow genomoncology/biomcp/.github/workflows/release.yml "
        f"--source-ref refs/tags/v{version} --source-digest {revision}\n```\n\n"
        "Replace ARTIFACT with the downloaded file. Compare the source commit above with the release's "
        "tag before accepting it. Private no-op rehearsals are unsigned and cannot pass attestation verification. "
        "Their retained release-subject-checksums artifact and candidate.json source commit identify the outputs, "
        "but do not provide signed provenance. For a signed public rehearsal, use the repository that ran the workflow for both "
        "repository arguments and replace --source-ref with the actual rehearsal ref. The source commit must match that run. Verification fails if the file, "
        "signer workflow or source commit differs. Stop if verification fails.\n\n"
        "The installer also checks the archive's SHA-256 checksum. Attestations identify the workflow "
        "and source that produced the bytes; they do not establish that the software is defect-free.\n"
    )


def subjects(args):
    root = args.artifacts.resolve()
    output = args.output.resolve()
    if not root.is_dir() or root == output or root in output.parents:
        raise ValueError("subjects require a staged directory and an output outside it")
    records = []
    for path in sorted(root.rglob("*")):
        name = path.relative_to(root).as_posix()
        if path.is_symlink() or any(char in name for char in "\r\n\\"):
            raise ValueError("unsafe staged attestation subject")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError("attestation subjects must be regular files")
        digest = hashlib.sha256()
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
        records.append(digest.hexdigest() + "  " + name + "\n")
    if not records:
        raise ValueError("staged attestation subjects must not be empty")
    with args.output.open("x") as target:
        target.writelines(records)


def stage(args):
    records = candidates(args)
    args.output.mkdir(parents=True, exist_ok=False)
    registry = json.loads((next(iter(records.values())) / "server.json").read_text())
    if registry.get("name") != "io.github.genomoncology/biomcp" or registry.get("version") != args.version:
        raise ValueError("registry identity differs from candidate")
    for tag, directory in records.items():
        copy_native(directory, args.output, tag, registry)
    (args.output / "server.json").write_text(json.dumps(registry, indent=2) + "\n")
    shutil.copy2(Path(__file__).parent / "install.sh", args.output / "install.sh")
    formula(args.output, args.version)
    plugin(args.plugin, args.output, args.version, next(iter(records.values())))
    zip_payload(args.output / "biomcp-plugin.zip")
    index = {"version": args.version, "revision": args.revision,
             "image": "ghcr.io/genomoncology/biomcp", "publication": "requires Ian's approval before pushing the release tag"}
    (args.output / "candidate.json").write_text(json.dumps(index, indent=2) + "\n")
    release_notes(args.output, args.version, args.revision)
    print(json.dumps(index))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    candidate = commands.add_parser("identity")
    candidate.add_argument("--event", choices=["push", "workflow_dispatch"], required=True)
    candidate.add_argument("--tag", required=True)
    candidate.add_argument("--revision", required=True)
    payloads = commands.add_parser("stage")
    for name in ["artifacts", "output", "plugin"]:
        payloads.add_argument("--" + name, type=Path, required=True)
    for name in ["version", "revision"]:
        payloads.add_argument("--" + name, required=True)
    attest = commands.add_parser("subjects")
    attest.add_argument("--artifacts", type=Path, required=True)
    attest.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    {"identity": identity, "stage": stage, "subjects": subjects}[args.command](args)


if __name__ == "__main__":
    try:
        main()
    except (KeyError, ValueError, OSError) as error:
        sys.exit(str(error))
