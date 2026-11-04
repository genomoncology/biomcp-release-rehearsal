"""Check article fixture reuse licenses; --list prints every covered capture."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


ALLOWED = {"CC0", "CC BY", "public domain"}


def recorded_licenses(value):
    if isinstance(value, dict):
        for key, item in value.items():
            if key == "license_code":
                yield item
            else:
                yield from recorded_licenses(item)
    elif isinstance(value, list):
        for item in value:
            yield from recorded_licenses(item)
    elif isinstance(value, str):
        # Read the article's own permissions, never licenses in cited papers.
        for permissions in re.findall(r"<permissions\b[^>]*>(.*?)</permissions>", value, re.S):
            for license_url in re.findall(r"https?://creativecommons.org/[^\s<>\"']+", permissions):
                if re.fullmatch(r"https?://creativecommons.org/licenses/by/[0-9.]+/", license_url):
                    yield "CC BY"
                elif re.fullmatch(r"https?://creativecommons.org/publicdomain/zero/[0-9.]+/", license_url):
                    yield "CC0"
                else:
                    yield license_url


def inventory(root):
    directory = root / "tests/replies/article"
    origins = json.loads((directory / "licenses.json").read_text())
    synthetic_pdfs = origins.pop("synthetic_pdf_sha256")
    for identity, origin in origins.items():
        if origin["license"] not in ALLOWED:
            raise ValueError(f"{identity}: disallowed article fixture license: {origin['license']}")
        evidence = json.loads((directory / origin["evidence"]).read_text())
        for license_code in recorded_licenses(evidence):
            if license_code not in ALLOWED:
                raise ValueError(f"{origin['evidence']}: disallowed article fixture license: {license_code}")
        if set(recorded_licenses(evidence)) != {origin["license"]}:
            raise ValueError(f"{identity}: license list disagrees with retained source evidence")
    rows, labels = [], {}
    for path in sorted(directory.rglob("*.json")):
        if path.name == "licenses.json":
            continue
        record = json.loads(path.read_text())
        url = record.get("url", "")
        pmc = record.get("source") == "pmc-oa"
        preprint = record.get("source") == "europepmc" and "/fullTextXML" in url
        publisher = record.get("source") == "semantic-scholar" and (
            "/doi/pdf/" in url or url.split("?", 1)[0].endswith(".pdf"))
        if not (pmc or preprint or publisher):
            continue
        relative = path.relative_to(directory).as_posix()
        licenses = list(recorded_licenses(record))
        for license_code in licenses:
            if license_code not in ALLOWED:
                raise ValueError(f"{relative}: disallowed article fixture license: {license_code}")
        identity = re.search(r"(?:PMC|PPR)[0-9]+", url)
        origin = origins.get(identity[0]) if identity else None
        # Only the retained generated PDF stub may bypass article licensing.
        # Resolve sidecars by recorder naming, never by the body's supplied path.
        generated_publisher = publisher and all(
            hashlib.sha256(path.with_name(f"{path.stem}-{index}.bin").read_bytes()).hexdigest()
            in synthetic_pdfs
            if exchange["body"]["kind"] == "sidecar"
            else "BioMCP synthetic" in str(exchange["body"].get("value", ""))
            for index, exchange in enumerate(record["exchanges"])
        )
        generated = any(part.startswith("synthetic") for part in path.parts) or generated_publisher
        # Empty listings and failed requests contain no article text or attachment.
        article = any(
            exchange["body"]["kind"] == "sidecar" or "<article" in str(exchange["body"].get("value", ""))
            for exchange in record["exchanges"]
        )
        if article and not origin and not generated_publisher:
            raise ValueError(f"{relative}: article capture has no recorded reuse license")
        label = origin["license"] if origin else "no article bytes"
        if generated:
            label = f"{label} source; synthetic case" if origin else "synthetic response"
        rows.append((relative, label))
        labels[path] = label
    # Use the recorder's file naming convention, never follow fixture paths.
    # The sidecar-escape case deliberately contains an invalid body path.
    for path in sorted(directory.rglob("*.bin")):
        owner = path.with_name(path.stem.rsplit("-", 1)[0] + ".json")
        if owner not in labels:
            raise ValueError(f"{path.relative_to(directory)}: unlisted article binary capture")
        rows.append((path.relative_to(directory).as_posix(), labels[owner]))
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--list", action="store_true")
    args = parser.parse_args()
    try:
        rows = inventory(args.root)
    except (ValueError, OSError) as error:
        print(error, file=sys.stderr)
        return 1
    if args.list:
        for path, license_code in rows:
            print(f"{path}\t{license_code}")
    print(f"Article fixture licenses qualify: {len(rows)} captures")
    return 0


if __name__ == "__main__":
    sys.exit(main())
