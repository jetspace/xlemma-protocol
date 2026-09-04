#!/usr/bin/env python3
"""Create a deterministic source archive outside the repository tree."""

from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo
from source_inventory import source_files

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT.parent / f"{ROOT.name}-v0.2.0.zip"


def main() -> None:
    files = source_files(ROOT) + [ROOT / "MANIFEST.sha256"]
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
