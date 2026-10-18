"""Repository-wide structural policy with deterministic diagnostics."""

from pathlib import Path
import os
import sys
import tomllib
import subprocess

from source import LANGUAGES, sloc
from boundary import uncovered

REPO = Path(__file__).resolve().parents[2]


def inspect(root: Path, base: Path, limits: dict) -> list[str]:
    """Inspect every source directory, including parents with no direct files."""
    label = root.relative_to(base).as_posix()
    errors = []
    if not root.is_dir() or root.is_symlink():
        return [f"{label}: root missing or not an inspectable directory"]
    try:
        sources = []
        def fail(error):
            errors.append(f"{label}: cannot inspect source: {error}")
        for parent, children, files in os.walk(root, onerror=fail):
            for child in list(children):
                if (Path(parent) / child).is_symlink():
                    errors.append(f"{label}: cannot inspect symlinked source directory: {Path(parent) / child}")
                    children.remove(child)
            sources.extend(Path(parent) / name for name in files if Path(name).suffix in LANGUAGES)
        sources.sort()
        directories = {root}
        for path in sources:
            if path.is_symlink() or not path.is_file():
                errors.append(f"{path.relative_to(base)}: source is not a regular file")
                continue
            directories.update(p for p in path.parents if p == root or root in p.parents)
            try:
                count = sloc(path.read_text(encoding="utf-8"), path.suffix)
            except (OSError, UnicodeError) as error:
                errors.append(f"{path.relative_to(base)}: cannot inspect source: {error}")
                continue
            if count > limits["sloc"]:
                errors.append(f"{path.relative_to(base)}: {count} SLOC; expected 0..{limits['sloc']}")
        for directory in sorted(directories):
            children = list(directory.iterdir())
            files = sum(p.is_file() and p.suffix in LANGUAGES for p in children)
            dirs = sum(p in directories for p in children if p.is_dir())
            rel = directory.relative_to(base)
            if not limits["files_min"] <= files <= limits["files_max"]:
                errors.append(f"{rel}: {files} source files; expected {limits['files_min']}..{limits['files_max']}")
            if dirs > limits["directories"]:
                errors.append(f"{rel}: {dirs} child directories; expected 0..{limits['directories']}")
    except (OSError, UnicodeError) as error:
        errors.append(f"{label}: cannot inspect source: {error}")
    return sorted(errors)


def audit(base: Path = REPO) -> list[str]:
    """Fail closed on invalid policy, missing roots, or unreadable sources."""
    try:
        config = tomllib.loads((base / "structural-governance.toml").read_text())
        roots = config["roots"]
        limits = config["limits"]
        if not roots or len(set(roots)) != len(roots):
            raise ValueError("roots must be nonempty and unique")
        if limits != {"directories": 3, "files_min": 1, "files_max": 4, "sloc": 200}:
            raise ValueError("policy must use the agreed 0..3 directories, 1..4 files, 200 SLOC")
        for root in roots:
            path = Path(root)
            if path.is_absolute() or ".." in path.parts:
                raise ValueError(f"root must remain within repository: {root}")
        errors = [error for root in roots for error in inspect(base / root, base, limits)]
        return sorted(errors + uncovered(base, roots))
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        return [f"structural-governance.toml: invalid policy: {error}"]


if __name__ == "__main__":
    violations = audit()
    for violation in violations:
        print(violation)
    print(f"Structural governance: {len(violations)} violation(s).")
    sys.exit(bool(violations))
