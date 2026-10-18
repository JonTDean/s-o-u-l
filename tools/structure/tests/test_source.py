"""Language-specific line counting regression tests."""

import unittest

from source import sloc


class SourceTests(unittest.TestCase):
    def test_rust_nested_comments_docs_and_raw_strings(self):
        text = '\n//! docs\n/* a /* nested */\nend */\nfn f() { // inline\nlet s = r#"/* string */"#;\n}\n'
        self.assertEqual(sloc(text, ".rs"), 3)

    def test_wgsl_comments_and_inline_code(self):
        text = '// docs\n/* outer /* nested */ end */\n@compute @workgroup_size(1)\nfn main() { /* inline */ }\n'
        self.assertEqual(sloc(text, ".wgsl"), 2)

    def test_powershell_block_comments_and_strings(self):
        text = '<# docs\nmore #>\n$x = "# string" # trailing\nWrite-Output $x\n'
        self.assertEqual(sloc(text, ".ps1"), 2)

    def test_python_documentation_and_multiline_strings(self):
        text = '"""module docs\nmore docs"""\n# comment\nx = """# literal\n/* literal */"""\n'
        self.assertEqual(sloc(text, ".py"), 2)

    def test_mixed_block_comment_and_code(self):
        for suffix in (".rs", ".wgsl"):
            with self.subTest(suffix=suffix):
                self.assertEqual(sloc('/* a */ let x = 1; /* b */\n', suffix), 1)

    def test_sloc_boundary(self):
        self.assertEqual(sloc('let x = 1; // comment\n' * 200, ".rs"), 200)
        self.assertEqual(sloc('let x = 1;\n' * 201, ".rs"), 201)
