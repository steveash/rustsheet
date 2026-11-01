"""Markdown helpers shared by the progress report's sections (SPEC-0022)."""

from __future__ import annotations

# The command that regenerates every generated page; printed in their headers and method notes.
REGENERATE = "make eval && make progress-report"


def pct(part: int, whole: int) -> str:
    """`part/whole` as a percentage with two decimals that never rounds up to 100 or down to 0."""
    if whole == 0:
        return "\N{EN DASH}"
    value = 100 * part / whole
    if 0 < part < whole:
        value = min(max(value, 0.01), 99.99)
    return f"{value:.2f}%"


# Markdown table cell text: a pipe would split the cell, `<`/`>` could open raw HTML, a backtick a
# code span, and a newline would end the row.
MD_CELL_ESCAPES = str.maketrans({"|": "\\|", "<": "&lt;", ">": "&gt;", "`": "&#96;", "\n": " ", "\r": " "})


def md_cell(text: str) -> str:
    """`text` made safe to put in one Markdown table cell (see MD_CELL_ESCAPES)."""
    return text.translate(MD_CELL_ESCAPES)


def table(header: list[str], rows: list[list[str]], right_from: int = 1) -> str:
    """A GitHub Markdown table; columns from `right_from` on are right-aligned (numbers)."""
    align = ["---" if i < right_from else "---:" for i in range(len(header))]
    body = [[md_cell(c) for c in line] for line in [header, *rows]]
    lines = [body[0], align, *body[1:]]
    return "\n".join("| " + " | ".join(line) + " |" for line in lines)


# Inline Markdown punctuation that could start emphasis, a link, an image or a heading, and the
# backslash itself, so a `\` in `text` cannot escape the character after it.
MD_INLINE = "\\*_[]()!#~"


def md_text(text: str) -> str:
    """`text` made safe to put in running Markdown: `&` written as an entity first (so a literal
    `&lt;` in `text` stays text, and md_cell's own entities are not escaped twice), inline
    punctuation and `\\` escaped, then md_cell (whose entities hold a `#` that must stay unescaped)."""
    amp = text.replace("&", "&amp;")
    return md_cell("".join(f"\\{c}" if c in MD_INLINE else c for c in amp))


def signed(delta: int) -> str:
    """`delta` with its sign: +3, \N{MINUS SIGN}2 or 0."""
    if delta > 0:
        return f"+{delta:,}"
    if delta < 0:
        return f"\N{MINUS SIGN}{-delta:,}"
    return "0"
