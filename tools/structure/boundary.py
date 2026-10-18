"""Ensure new first-party source cannot silently fall outside the governed roots."""

from pathlib import Path
import subprocess

from source import LANGUAGES


def uncovered(base: Path, roots: list[str]) -> list[str]:
    """Inspect tracked and nonignored new files; build/dependency output stays out."""
    result = subprocess.run(
        ["git", "-C", str(base), "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        check=True, capture_output=True,
    )
    boundaries = [Path(root) for root in roots]
    errors = []
    for name in result.stdout.decode().split("\0"):
        path = Path(name)
        if path.suffix not in LANGUAGES or not (base / path).exists():
            continue
        if not any(path.is_relative_to(root) for root in boundaries):
            errors.append(f"{path}: first-party source is outside governed roots")
    return sorted(errors)
