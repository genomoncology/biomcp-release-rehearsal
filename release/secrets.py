#!/usr/bin/env python3
"""Run the pinned secret scanner and disclose only finding paths and rule names."""
import argparse
import gzip
import hashlib
import io
import json
import lzma
import os
from pathlib import Path, PurePosixPath
import platform
import re
import stat
import subprocess
import tarfile
import tempfile
import urllib.request
import zipfile

# Official release assets and SHA-256 digests:
# https://github.com/gitleaks/gitleaks/releases
# CLI: https://github.com/gitleaks/gitleaks/blob/master/README.md#usage
if __package__:
    from .toolchain import manifest
else:
    from toolchain import manifest

SCANNER = manifest()["gitleaks"]
VERSION = SCANNER["version"]
ARCHIVES = {tuple(key.split("/")): value for key, value in SCANNER["archives"].items()}



def scanner(cache, scratch):
    suffix, digest = ARCHIVES[(platform.system(), platform.machine())]
    name = "gitleaks_" + VERSION + "_" + suffix + ".tar.gz"
    cache.mkdir(parents=True, exist_ok=True)
    archive = cache / name
    if not archive.exists():
        url = "https://github.com/gitleaks/gitleaks/releases/download/v" + VERSION + "/" + name
        with urllib.request.urlopen(url, timeout=120) as response:
            payload = response.read()
        if hashlib.sha256(payload).hexdigest() != digest:
            raise ValueError("scanner download checksum mismatch")
        # Atomic replacement also permits concurrent gates using the same cache.
        with tempfile.NamedTemporaryFile(dir=cache, delete=False) as output:
            output.write(payload)
        Path(output.name).replace(archive)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != digest:
        raise ValueError("scanner cache checksum mismatch")
    binary = scratch / "gitleaks"
    with tarfile.open(archive) as bundle:
        member = bundle.getmember("gitleaks")
        if not member.isfile():
            raise ValueError("scanner executable is not a regular file")
        binary.write_bytes(bundle.extractfile(member).read())
    binary.chmod(0o700)
    return binary


# These bounds fail closed. They include uncompressed container layers and all
# historical blobs; they never tell the scanner to silently skip large input.
MAX_FILE = 512 * 1024 * 1024
MAX_TOTAL = 8 * 1024 * 1024 * 1024
MAX_DEPTH = 16
MAX_FILES = 200000
ARCHIVE_SUFFIXES = (".zip", ".whl", ".mcpb", ".tar", ".gz", ".tgz", ".7z", ".rar", ".bz2", ".xz", ".zst")


HISTORY_IGNORE = Path(__file__).with_name("secrets.ignore.jsonl")
HISTORY_SCOPE = ("--single-worktree", "--all", "HEAD")


def git(target, env, *args):
    return subprocess.check_output(["git", "--no-replace-objects", "-C", str(target), *args],
                                   env=env, stderr=subprocess.DEVNULL)


def inputs(mode, target, env, since=None, progress=None):
    if mode != "history":
        def unreadable(error):
            raise error
        for directory, directories, files in os.walk(target, followlinks=False, onerror=unreadable):
            for name in sorted(directories + files):
                path = Path(directory) / name
                # Only fixed publication labels can cross the diagnostic boundary.
                # Nested paths and arbitrary filesystem names may contain secrets.
                if progress is not None:
                    progress["artifact"] = (name if name in ("biomcp-container-amd64.tar", "biomcp-container-arm64.tar")
                                            else "untrusted-artifact-name")
                if path.is_symlink() or not (path.is_file() or path.is_dir()):
                    raise ValueError("nonregular artifact")
                if path.is_file():
                    with path.open("rb") as source:
                        yield path.relative_to(target).as_posix(), source.read(MAX_FILE + 1)
        return
    if git(target, env, "rev-parse", "--is-shallow-repository").strip() != b"false":
        raise ValueError("secret history scan requires a complete checkout")
    # Enumerate actual reachable blobs, not patches: merge-only additions, binary
    # attributes and default diff suppression must not hide historical content.
    # Keep named refs and this HEAD without importing detached sibling jobs.
    scope = list(HISTORY_SCOPE)
    if since is not None:
        if not re.fullmatch("[a-f0-9]{40}", since):
            raise ValueError("secret baseline must be a full commit hash")
        git(target, env, "merge-base", "--is-ancestor", since, "HEAD")
        scope = ["HEAD", "^" + since]
    objects = git(target, env, "rev-list", "--objects", *scope).splitlines()
    names = {}
    for row in objects:
        oid, _, name = row.partition(b" ")
        if len(oid) == 40:
            names.setdefault(oid, name.decode("utf-8", errors="replace"))
    ids = git(target, env, "rev-list", "--objects", *scope, "--no-object-names")
    types = subprocess.check_output(["git", "--no-replace-objects", "-C", str(target), "cat-file", "--batch-check"],
                                    input=ids, env=env, stderr=subprocess.DEVNULL)
    with subprocess.Popen(["git", "--no-replace-objects", "-C", str(target), "cat-file", "--batch"],
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env) as process:
        try:
            for row in types.splitlines():
                oid, kind, size = row.split()
                if kind != b"blob":
                    continue
                if int(size) > MAX_FILE:
                    raise ValueError("historical blob exceeds screening limit")
                process.stdin.write(oid + b"\n")
                process.stdin.flush()
                header = process.stdout.readline().split()
                if header != [oid, kind, size]:
                    raise ValueError("historical blob unavailable")
                payload = process.stdout.read(int(size))
                if len(payload) != int(size) or process.stdout.read(1) != b"\n":
                    raise ValueError("historical blob incomplete")
                yield names.get(oid) or "git-blob/" + oid.decode(), payload
        finally:
            process.stdin.close()
            process.stdout.close()
            if process.wait():
                raise ValueError("historical blob reader failed")


def alternatives_metadata(name, payload):
    # dpkg names each alternatives database after its link group, including
    # groups such as builtins.7.gz. These are text records, not gzip members.
    # Only a nested canonical database path with a text header qualifies;
    # gzip magic still takes precedence and every byte is screened above.
    parts = name.split("!")
    path = PurePosixPath(parts[-1])
    return (len(parts) > 1 and path.parent == PurePosixPath("var/lib/dpkg/alternatives")
            and payload.startswith((b"auto\n", b"manual\n"))
            and all(byte in (9, 10, 13) or 32 <= byte <= 126 for byte in payload))


def prepare(target, mode, root, env, since=None, progress=None):
    directory = root / "inputs"
    directory.mkdir()
    mapping, total, members = {}, 0, 0
    def add(name, payload, depth=0):
        nonlocal total, members
        total += len(payload)
        if len(payload) > MAX_FILE or total > MAX_TOTAL or depth > MAX_DEPTH or len(mapping) >= MAX_FILES:
            raise ValueError("artifact screening limit exceeded")
        key = str(len(mapping)) + ".txt"
        mapping[key] = {"file": name, **origin}
        # Neutral names remove default path allowlists; a plain-text prefix also
        # prevents MIME detection from silently skipping executable bytes. UTF-8
        # replacement preserves every ASCII credential byte and its boundaries.
        (directory / key).write_text("screened content\n" + payload.decode("utf-8", errors="replace"), encoding="utf-8")
        suffix = name.lower()
        def opaque():
            print(json.dumps({"file": name, "rule": "opaque-history-blob-decoded-coverage-unavailable", "level": "info"}))
        def member_name(member):
            path = PurePosixPath(member.replace("\\", "/"))
            if path.is_absolute() or ".." in path.parts or (path.parts and ":" in path.parts[0]):
                raise ValueError("escaping archive member")
            return name + "!" + member
        if payload.startswith((b"PK\x03\x04", b"PK\x05\x06", b"PK\x07\x08")) or suffix.endswith((".zip", ".whl", ".mcpb")):
            def contents():
                nonlocal members
                try:
                    try:
                        archive = zipfile.ZipFile(io.BytesIO(payload))
                    except ValueError:
                        # Python 3.9 reports an impossible central-directory
                        # offset as ValueError rather than BadZipFile. Limit
                        # this compatibility case to construction, never our
                        # member, escape or expansion-limit checks below.
                        if mode == "history" and not suffix.endswith(ARCHIVE_SUFFIXES):
                            opaque()
                            return
                        raise
                    with archive:
                        for member in archive.infolist():
                            members += 1
                            if members > MAX_FILES:
                                raise ValueError("too many archive members")
                            nested = member_name(member.filename)
                            if member.is_dir():
                                if member.file_size:
                                    raise ValueError("directory member carries unscanned content")
                                continue
                            if member.file_size > MAX_FILE or total + member.file_size > MAX_TOTAL:
                                raise ValueError("expanded archive exceeds screening limit")
                            kind = stat.S_IFMT(member.external_attr >> 16)
                            if kind not in (0, stat.S_IFREG, stat.S_IFLNK):
                                raise ValueError("unsupported archive member")
                            try:
                                with archive.open(member) as source:
                                    yield (nested + "!link-target" if kind == stat.S_IFLNK else nested,
                                           source.read(MAX_FILE + 1))
                            except RuntimeError:
                                if mode == "history" and not suffix.endswith(ARCHIVE_SUFFIXES):
                                    opaque()
                                    return
                                raise
                except zipfile.BadZipFile:
                    # Historical parser probes may deliberately resemble ZIP magic.
                    # Their bytes were staged above. Declared archives and current
                    # publication artifacts must still fail on malformed structure.
                    if mode == "history" and not suffix.endswith(ARCHIVE_SUFFIXES):
                        opaque()
                        return
                    raise
            for nested, content in contents():
                # A nested declared archive keeps its own strict error policy;
                # its failure must not become an outer opaque-blob fallback.
                add(nested, content, depth + 1)
        elif payload.startswith(b"\x1f\x8b") or (suffix.endswith((".gz", ".tgz")) and not alternatives_metadata(name, payload)):
            with gzip.GzipFile(fileobj=io.BytesIO(payload)) as source:
                add(name + "!uncompressed.tar" if suffix.endswith((".tar.gz", ".tgz")) else name + "!uncompressed", source.read(MAX_FILE + 1), depth + 1)
        elif suffix.endswith(".tar") or (len(payload) > 262 and payload[257:262] == b"ustar"):
            with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
                for member in archive:
                    members += 1
                    if members > MAX_FILES:
                        raise ValueError("too many archive members")
                    nested = member_name(member.name)
                    if member.isdir():
                        if member.size:
                            raise ValueError("directory member carries unscanned content")
                        continue
                    if member.issym() or member.islnk():
                        # Container layers contain links. Scan their metadata,
                        # never dereference them or extract their target paths.
                        add(nested + "!link-target", member.linkname.encode(), depth + 1)
                    elif member.isfile():
                        if member.size > MAX_FILE or total + member.size > MAX_TOTAL:
                            raise ValueError("expanded archive exceeds screening limit")
                        with archive.extractfile(member) as source:
                            add(nested, source.read(MAX_FILE + 1), depth + 1)
                    else:
                        raise ValueError("unsupported archive member")
        elif payload.startswith(b"\xfd7zXZ\x00") or suffix.endswith(".xz"):
            with lzma.LZMAFile(io.BytesIO(payload)) as source:
                add(name + "!uncompressed.tar" if suffix.endswith(".tar.xz") else name + "!uncompressed",
                    source.read(MAX_FILE + 1), depth + 1)
        elif suffix.endswith((".7z", ".rar", ".bz2", ".zst")) or payload.startswith((b"7z\xbc\xaf\x27\x1c", b"Rar!", b"BZh", b"\xfd7zXZ\x00", b"\x28\xb5\x2f\xfd")):
            if mode == "history" and not suffix.endswith(ARCHIVE_SUFFIXES):
                opaque()
                return
            raise ValueError("unsupported archive compression")
    for name, payload in inputs(mode, target, env, since, progress):
        blob = hashlib.sha1(b"blob " + str(len(payload)).encode() + b"\0" + payload).hexdigest() if mode == "history" else ""
        origin = {"blob": blob, "git_file": name}
        add(name, payload)
    return directory, mapping


class HistoryBindingError(ValueError):
    """A reviewed binding failed with a fixed public cause and ledger row."""

    def __init__(self, code, row):
        self.code, self.row = code, row


def history_ignores(target, env, mapping, staging):
    if not HISTORY_IGNORE.exists():
        return []
    index = {}
    for key, item in mapping.items():
        index.setdefault((item["blob"], item["file"]), []).append(key)
    translated, reachable = [], None
    for number, line in enumerate(HISTORY_IGNORE.read_text().splitlines(), 1):
        row = json.loads(line)
        if (set(row) != {"commit", "file", "blob", "rule", "line", "reason"}
                or not all(re.fullmatch("[a-f0-9]{40}", row[k]) for k in ["commit", "blob"])
                or not re.fullmatch("[A-Za-z0-9_-]+", row["rule"])
                or type(row["line"]) is not int or row["line"] < 1
                or not isinstance(row["reason"], str) or not row["reason"].strip() or "\n" in row["reason"]):
            raise ValueError("invalid reviewed history exception")
        keys = index.get((row["blob"], row["file"]), [])
        if not keys:
            continue
        if reachable is None:
            reachable = set(git(target, env, "rev-list", *HISTORY_SCOPE).decode().splitlines())
        if row["commit"] not in reachable:
            raise HistoryBindingError("history-ignore-unreachable-commit", number)
        try:
            blob = git(target, env, "rev-parse", row["commit"] + ":" + row["file"]).decode().strip()
        except subprocess.CalledProcessError:
            raise HistoryBindingError("history-ignore-wrong-blob", number) from None
        if blob != row["blob"]:
            raise HistoryBindingError("history-ignore-wrong-blob", number)
        for key in keys:
            if mapping[key]["git_file"] != row["file"]:
                raise ValueError("history exception must name its actual Git path")
            payload = (staging / key).read_bytes()
            lines = payload.count(b"\n") + (not payload.endswith(b"\n")) - 1
            if row["line"] > lines:
                raise HistoryBindingError("history-ignore-wrong-line", number)
            translated.append(str(staging / key) + ":" + row["rule"] + ":" + str(row["line"] + 1))
    return translated


def scanner_config(mode, mapping, staging):
    text = '[extend]\nuseDefault = true\n'
    if mode == "history":
        return text
    # Match-specific rules cannot exempt another finding on the same line.
    # History uses immutable reviewed bindings instead of these fixture forms.
    groups = [
        (lambda p: p == "tests/parity/sequences.json" or p.startswith("tests/replies/trial/"),
         [r'^(?:nextPageToken|pageToken)"\s*:\s*"[A-Za-z0-9_-]{39}"$', r'^pageToken=[A-Za-z0-9_-]{39}["\s;]?$']),
        (lambda p: p == "crates/biomcp/tests/runner/study_catalog.rs", [r'^key\s*=\s*"[a-f0-9]{64}"$']),
        (lambda p: p.startswith(("tests/fixtures/", "tests/replies/")),
         [r'^(?:request_key|request_sha256|capture_digest|body_sha256)"\s*:\s*"[a-f0-9]{64}"$']),
    ]
    rule_added = False
    for selected, patterns in groups:
        paths = ["^" + re.escape(str(staging / key)) + "$" for key, item in mapping.items() if selected(item["file"])]
        if paths:
            if not rule_added:
                text += '\n[[rules]]\nid = "generic-api-key"\n'
                rule_added = True
            text += '[[rules.allowlists]]\ncondition = "AND"\nregexTarget = "match"\n'
            text += "paths = " + json.dumps(paths) + "\nregexes = " + json.dumps(patterns) + "\n"
    return text


def findings(report, mapping, staging):
    # This report contains coordinates and identities only, never source lines,
    # matches or captured secrets. Public output remains file plus rule.
    for row in report.read_text().splitlines():
        file, rule, first, last, column, end = row.split("\t")
        key = Path(file).name
        yield ({**mapping[key], "rule": rule, "line": int(first) - 1},
               staging / key, (int(first), int(last), int(column), int(end)))


ERRORS = (OSError, ValueError, KeyError, subprocess.SubprocessError,
          tarfile.TarError, zipfile.BadZipFile, lzma.LZMAError, RuntimeError, EOFError, TypeError)


class ScreeningError(Exception):
    """Only fixed classifications and artifact coordinates cross this boundary."""

    def __init__(self, category, progress):
        self.diagnostic = {"category": category, **progress}


def scan(mode, target, cache, since=None):
    progress = {"stage": "target-validation", "artifact": "scan-target"}
    try:
        return screen(mode, target, cache, since, progress)
    except HistoryBindingError:
        raise
    except ERRORS as error:
        if isinstance(error, gzip.BadGzipFile):
            category = "validation"
        elif isinstance(error, OSError):
            category = "filesystem"
        elif progress["stage"] in ("scanner-execution", "scanner-report"):
            category = "scanner"
        elif isinstance(error, (ValueError, tarfile.TarError, zipfile.BadZipFile, lzma.LZMAError, EOFError)):
            category = "validation"
        else:
            category = "preparation"
        raise ScreeningError(category, progress) from None


def screen(mode, target, cache, since, progress):
    if target.is_symlink():
        raise ValueError("symbolic-link scan target")
    target = target.resolve(strict=True)
    if not target.is_dir():
        raise ValueError("scan target must be a directory")
    with tempfile.TemporaryDirectory(prefix="biomcp-secret-scan-") as directory:
        root = Path(directory).resolve()
        if __package__:
            from .smoke import environment
        else:
            from smoke import environment
        env = {**environment(root), "PATH": os.pathsep.join(("/usr/local/bin", os.defpath)), "GIT_CONFIG_NOSYSTEM": "1", "GIT_TERMINAL_PROMPT": "0",
               "GIT_NO_LAZY_FETCH": "1", "GIT_GRAFT_FILE": os.devnull, "GIT_CONFIG_COUNT": "2",
               "GIT_CONFIG_KEY_0": "core.hooksPath", "GIT_CONFIG_VALUE_0": os.devnull,
               "GIT_CONFIG_KEY_1": "core.fsmonitor", "GIT_CONFIG_VALUE_1": "false"}
        progress["stage"] = "scanner-preparation"
        binary = scanner(cache.resolve(), root)
        progress["stage"] = "artifact-preparation"
        staging, mapping = prepare(target, mode, root, env, since, progress)
        progress.update(stage="scanner-configuration", artifact="scan-target")
        config, template, ignore, report = [root / name for name in ["config.toml", "report.tmpl", "ignore", "report.txt"]]
        digests = set()
        if mode == "artifacts":
            if __package__:
                from .public_fixtures import reviewed, recognized
            else:
                from public_fixtures import reviewed, recognized
            digests = reviewed()
        config.write_text(scanner_config(mode, mapping, staging))
        template.write_text(r'{{range .}}{{printf "%s\t%s\t%d\t%d\t%d\t%d\n" .File .RuleID .StartLine .EndLine .StartColumn .EndColumn}}{{end}}')
        ignore.write_text("\n".join(history_ignores(target, env, mapping, staging)) if mode == "history" else "")
        command = [str(binary), "dir", str(staging), "--config", str(config), "--gitleaks-ignore-path", str(ignore),
                   "--ignore-gitleaks-allow", "--redact=100", "--no-banner", "--no-color", "--log-level", "error",
                   "--report-format", "template", "--report-template", str(template), "--report-path", str(report)]
        # Never retain raw scanner diagnostics or JSON/SARIF finding reports.
        progress["stage"] = "scanner-execution"
        result = subprocess.run(command, cwd=root, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        if result.stderr:
            raise ValueError("secret scanner could not screen every input")
        if result.returncode == 1 and report.is_file():
            progress["stage"] = "scanner-report"
            rejected = False
            reported = 0
            for finding, path, coordinates in findings(report, mapping, staging):
                reported += 1
                if mode == "artifacts" and finding["rule"] == "private-key" and recognized(path, coordinates, digests):
                    continue
                rejected = True
                print(json.dumps({"file": finding["file"], "rule": finding["rule"]}))
            if not reported:
                raise ValueError("secret scanner finding report was empty")
            if rejected:
                return 1
            print("Secret scan passed")
            return 0
        if result.returncode:
            raise ValueError("secret scanner failed; diagnostic output was suppressed")
        print("Secret scan passed")
        return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["history", "artifacts"])
    parser.add_argument("target", type=Path)
    parser.add_argument("--cache", type=Path, default=Path("target/security-tools"))
    parser.add_argument("--since", help="previously scanned ancestor commit; omit for full history")
    args = parser.parse_args(argv)
    try:
        raise SystemExit(scan(args.mode, args.target, args.cache, args.since))
    except HistoryBindingError as error:
        parser.exit(2, json.dumps({"code": error.code, "row": error.row}) + "\n")
    except ScreeningError as error:
        parser.exit(2, json.dumps(error.diagnostic) + "\n")


if __name__ == "__main__":
    main()
