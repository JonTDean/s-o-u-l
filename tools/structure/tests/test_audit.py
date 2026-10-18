"""Boundary, traversal, failure, and diagnostic regression tests."""

from pathlib import Path
from tempfile import TemporaryDirectory
import os
import unittest
from unittest.mock import patch

from audit import REPO, audit, inspect
from boundary import uncovered
from types import SimpleNamespace

LIMITS = {"directories": 3, "files_min": 1, "files_max": 4, "sloc": 200}


class AuditTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "src"
        self.root.mkdir()

    def put(self, name, text="fn f() {}\n"):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def report(self):
        return inspect(self.root, self.base, LIMITS)

    def test_one_and_four_files_are_inclusive(self):
        self.put("a.rs")
        self.assertEqual(self.report(), [])
        for name in ("b.rs", "c.rs", "d.rs"):
            self.put(name)
        self.assertEqual(self.report(), [])
        self.put("e.rs")
        self.assertEqual(self.report(), ["src: 5 source files; expected 1..4"])

    def test_zero_source_files_fails(self):
        self.assertEqual(self.report(), ["src: 0 source files; expected 1..4"])

    def test_three_and_four_child_directories(self):
        self.put("mod.rs")
        for name in ("a", "b", "c"):
            self.put(f"{name}/mod.rs")
        self.assertEqual(self.report(), [])
        self.put("d/mod.rs")
        self.assertEqual(self.report(), ["src: 4 child directories; expected 0..3"])

    def test_recursive_parents_and_source_line_limit(self):
        self.put("mod.rs", "fn f() {}\n" * 200)
        self.put("layer/leaf/mod.rs", "fn g() {}\n" * 201)
        self.assertEqual(self.report(), [
            "src/layer/leaf/mod.rs: 201 SLOC; expected 0..200",
            "src/layer: 0 source files; expected 1..4",
        ])

    def test_metadata_and_binary_assets_are_outside_source_boundary(self):
        self.put("a.rs")
        self.put("data/scene.json", "{}")
        self.assertEqual(self.report(), [])

    def test_missing_root_and_unreadable_file_fail(self):
        self.assertIn("root missing", inspect(self.base / "missing", self.base, LIMITS)[0])
        self.put("a.rs")
        with patch.object(Path, "read_text", side_effect=PermissionError("denied")):
            self.assertIn("cannot inspect", self.report()[0])

    def test_directory_traversal_errors_fail_closed(self):
        def denied(root, onerror):
            onerror(PermissionError("denied subtree"))
            return []
        with patch("audit.os.walk", side_effect=denied):
            self.assertTrue(any("denied subtree" in error for error in self.report()))

    def test_symlinked_source_cannot_escape_the_boundary(self):
        self.put("a.rs")
        (self.root / "escape").symlink_to(self.base, target_is_directory=True)
        self.assertIn("symlinked source directory", self.report()[0])

    def test_all_errors_are_sorted_and_reported(self):
        self.put("z/deep/a.rs", "fn f() {}\n" * 201)
        errors = self.report()
        self.assertEqual(errors, sorted(errors))
        self.assertEqual(len(errors), 3)

    def test_repository_resolves_independently_of_cwd(self):
        previous = Path.cwd()
        try:
            os.chdir(self.base)
            self.assertEqual(REPO / "Cargo.toml", Path(__file__).resolve().parents[3] / "Cargo.toml")
        finally:
            os.chdir(previous)

    def test_missing_and_weakened_policy_fail_closed(self):
        self.assertTrue(audit(self.base))
        (self.base / "structural-governance.toml").write_text('roots = ["src"]\n[limits]\nsloc = 999\n')
        self.assertIn("agreed", audit(self.base)[0])

    def test_new_source_outside_roots_is_reported(self):
        self.put("inside.rs")
        (self.base / "outside.rs").write_text("fn f() {}")
        result = SimpleNamespace(stdout=b"src/inside.rs\0outside.rs\0deleted.rs\0README.md\0")
        with patch("boundary.subprocess.run", return_value=result):
            self.assertEqual(uncovered(self.base, ["src"]), ["outside.rs: first-party source is outside governed roots"])

    def test_compliant_policy_from_another_working_directory(self):
        self.put("a.rs")
        policy = 'roots = ["src"]\n[limits]\ndirectories = 3\nfiles_min = 1\nfiles_max = 4\nsloc = 200\n'
        (self.base / "structural-governance.toml").write_text(policy)
        previous = Path.cwd()
        try:
            os.chdir("/tmp")
            with patch("audit.uncovered", return_value=[]):
                self.assertEqual(audit(self.base), [])
        finally:
            os.chdir(previous)

    def test_unreadable_source_does_not_hide_other_violations(self):
        self.put("a.rs")
        (self.root / "a.rs").write_bytes(b"\xff")
        self.put("z.rs", "fn f() {}\n" * 201)
        errors = self.report()
        self.assertEqual(len(errors), 2)
        self.assertIn("cannot inspect source", errors[0])
        self.assertEqual(errors[1], "src/z.rs: 201 SLOC; expected 0..200")
