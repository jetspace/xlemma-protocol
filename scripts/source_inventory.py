"""Shared release inventory; local credentials and runtime data are not source."""

from pathlib import Path

EXCLUDED_PARTS = {
    ".git", ".agents", ".codex", ".venv", "venv", "node_modules", "target",
    ".lake", "out", "cache", "__pycache__", "artifacts", "secrets",
}
EXCLUDED_SUFFIXES = {".pem", ".key", ".p12", ".pfx", ".log", ".tmp", ".swp", ".zip", ".pyc", ".pyo"}


def source_files(root: Path) -> list[Path]:
    files = []
    for path in root.rglob("*"):
        relative = path.relative_to(root)
        if any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        if path.name == ".DS_Store" or path.suffix.lower() in EXCLUDED_SUFFIXES:
            continue
        if any(part.startswith(".env") and part != ".env.example" for part in relative.parts):
            continue
        if path.is_symlink():
            raise RuntimeError(f"refusing symlink in source inventory: {relative}")
        if path.is_file() and relative != Path("MANIFEST.sha256"):
            files.append(path)
    return sorted(files)
