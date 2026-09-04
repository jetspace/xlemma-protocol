#!/usr/bin/env python3
"""Create a deterministic source archive outside the repository tree."""

from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT.parent / f"{ROOT.name}-v0.2.0.zip"
EXCLUDED_PARTS = {"target", ".git", ".lake", "out", "cache", "__pycache__"}


def main() -> None:
    files = [
        path
        for path in ROOT.rglob("*")
        if path.is_file() and not any(part in EXCLUDED_PARTS for part in path.parts)
    ]
    with ZipFile(OUTPUT, "w", ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(files):
            relative = Path(ROOT.name) / path.relative_to(ROOT)
            info = ZipInfo(str(relative).replace("\\", "/"))
            info.date_time = (2026, 9, 3, 0, 0, 0)
            info.compress_type = ZIP_DEFLATED
            mode = 0o100755 if path.stat().st_mode & 0o111 else 0o100644
            info.external_attr = mode << 16
            archive.writestr(info, path.read_bytes())
    print(OUTPUT)


if __name__ == "__main__":
    main()
