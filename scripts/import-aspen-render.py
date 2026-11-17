#!/usr/bin/env python3
"""Import only the immutable, anonymous-public Aspen render contract closure.

Run from the repository root. An optional archive path avoids downloading again;
it must have the same pinned archive identity. Imported bytes are never patched.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import urllib.request

REVISION = "afc8308bff9a8c50b2315cb4a17c9454252d7c45"
URL = f"https://codeload.github.com/brittonr/aspen/tar.gz/{REVISION}"
ARCHIVE_BLAKE3 = "a6307998de7d69f737e1ce39465c63f6742aa4612979321597a304b89771363d"
PREFIX = f"aspen-{REVISION}/"
DEST = Path("vendor-cargo-deps/aspen-render")
CRATES = ("core", "batch-core", "codec", "render")
WORKSPACE = '''# Lattice-owned sparse import wrapper; producer files below remain byte-exact.
[workspace]
members = ["core", "batch-core", "codec", "render"]
resolver = "2"
[workspace.package]
edition = "2021"
license = "AGPL-3.0-or-later"
rust-version = "1.94"
[workspace.dependencies]
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
serde_json = "1"
'''

def digest(data):
    return subprocess.run(["b3sum", "--no-names"], input=data, capture_output=True, check=True).stdout.decode().strip()

def main():
    if len(sys.argv) > 2:
        raise SystemExit("usage: python3 scripts/import-aspen-render.py [EXACT_ARCHIVE]")
    data = Path(sys.argv[1]).read_bytes() if len(sys.argv) == 2 else urllib.request.urlopen(URL).read()
    if digest(data) != ARCHIVE_BLAKE3:
        raise SystemExit("public producer archive identity mismatch")
    payloads = {}
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        for member in archive.getmembers():
            if member.isdir() and member.name == PREFIX.rstrip("/"):
                continue
            if not member.name.startswith(PREFIX):
                raise SystemExit("archive prefix mismatch")
            relative = member.name[len(PREFIX):]
            base = "pilots/orchestration/"
            selected = relative in ("LICENSE", "LICENSE-AGPL", "LICENSE-AGPL-3.0", "COPYING")
            selected |= any(relative.startswith(base + crate + "/") for crate in CRATES)
            if not selected or member.isdir():
                continue
            if not member.isfile() or ".." in Path(relative).parts:
                raise SystemExit("non-regular archive selection")
            output = "source/" + relative[len(base):] if relative.startswith(base) else relative
            payloads[output] = archive.extractfile(member).read()
    if not any(name.startswith("LICENSE") or name == "COPYING" for name in payloads):
        raise SystemExit("missing upstream license")
    payloads["source/Cargo.toml"] = WORKSPACE.encode()
    provenance = {"repository": "https://github.com/brittonr/aspen", "revision": REVISION,
                  "archive_url": URL, "archive_blake3": ARCHIVE_BLAKE3,
                  "selection": ["pilots/orchestration/" + crate for crate in CRATES],
                  "wrapper": "source/Cargo.toml is Lattice-owned sparse workspace metadata",
                  "files": {name: digest(value) for name, value in sorted(payloads.items())}}
    payloads["SOURCE.json"] = (json.dumps(provenance, indent=2) + "\n").encode()
    # Existing identical snapshots are safe; refuse to overwrite any changed bytes.
    for name, value in payloads.items():
        target = DEST / name
        if target.exists() and target.read_bytes() != value:
            raise SystemExit(f"refusing to overwrite changed imported file: {target}")
    for name, value in payloads.items():
        target = DEST / name
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(value)
    print(f"Imported {REVISION}: {len(payloads)} files; anonymous public archive {ARCHIVE_BLAKE3}")

if __name__ == "__main__":
    main()
