#!/usr/bin/env python3
"""Generate the deterministic SHA-256 source manifest for xLemma."""

from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "MANIFEST.sha256"
EXCLUDED_PARTS = {".git", "target", ".lake", "out", "cache", "__pycache__"}


def source_files() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*"):
        if path.is_symlink():
            raise RuntimeError(f"refusing symlink in source manifest: {path.relative_to(ROOT)}")
        if (
            path.is_file()
            and path != MANIFEST
            and not any(part in EXCLUDED_PARTS for part in path.relative_to(ROOT).parts)
        ):
            files.append(path)
    return sorted(files)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    lines = [f"{sha256(path)}  {path.relative_to(ROOT).as_posix()}" for path in source_files()]
    MANIFEST.write_text("\n".join(lines) + "\n")
    print(f"wrote {MANIFEST} with {len(lines)} entries")


if __name__ == "__main__":
    main()
