"""Measure source lines using language-aware tokenization."""

from pygments import lex
from pygments.lexers import get_lexer_by_name
from pygments.token import Comment, String

LANGUAGES = {".rs": "rust", ".wgsl": "wgsl", ".ps1": "powershell", ".py": "python"}


def sloc(source: str, suffix: str) -> int:
    """Count nonblank code lines, preserving strings and excluding documentation."""
    lexer = get_lexer_by_name(LANGUAGES[suffix], stripnl=False, ensurenl=False)
    code_lines: set[int] = set()
    line = 1
    for kind, value in lex(source, lexer):
        is_code = kind not in Comment and kind not in String.Doc
        parts = value.split("\n")
        if is_code:
            code_lines.update(line + i for i, part in enumerate(parts) if part.strip())
        line += len(parts) - 1
    return len(code_lines)
