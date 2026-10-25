"""The block markers of docs/progress.md, and judgment-block preservation (SPEC-0022).

The page is a sequence of named blocks between HTML comment markers:

    <!-- progress:generated status --> ... <!-- /progress:generated -->
    <!-- progress:judgment risks --> ... <!-- /progress:judgment -->

The generator writes every generated block in full. The steward writes the judgment blocks; the
generator copies the text between a judgment block's two markers into the new page byte for byte.
Anything it cannot account for stops the run with a `ProgressReportError` instead of being
dropped: a block name it does not know, a duplicate, nested or unbalanced markers, a malformed or
unterminated marker comment, and any `progress:` marker inside judgment text.
"""

from __future__ import annotations

import re
from collections.abc import Collection
from dataclasses import dataclass

from rustsheet_tools.progress_report.model import COMMIT_SHA, RUN_DATE, ProgressReportError, Run

GENERATED = "generated"
JUDGMENT = "judgment"
# One HTML comment, shortest match; a comment that mentions `progress:` must be a marker.
COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)
MARKER = re.compile(r"<!-- (/?)progress:(generated|judgment)(?: ([a-z][a-z0-9-]*))? -->")
MARKER_WORD = "progress:"
# The steward's first line in a judgment block: the run (date and short commit) it was written for.
AS_OF = re.compile(rf"<!-- as-of: ({RUN_DATE.pattern}) ({COMMIT_SHA.pattern}) -->")
# The text of a judgment block the steward has not written yet.
PLACEHOLDER = "\n_Not written yet: the steward writes this block (SPEC-0022)._\n"


def open_marker(kind: str, name: str) -> str:
    """The marker that opens block `name` of `kind`."""
    return f"<!-- {MARKER_WORD}{kind} {name} -->"


def close_marker(kind: str) -> str:
    """The marker that closes a block of `kind`."""
    return f"<!-- /{MARKER_WORD}{kind} -->"


@dataclass(frozen=True)
class Block:
    """One block of an existing page: its kind, name and the exact text between its markers,
    which is `page[start:end]`."""

    kind: str
    name: str
    text: str
    start: int
    end: int


@dataclass(frozen=True)
class _Marker:
    closing: bool
    kind: str
    name: str | None
    start: int
    end: int
    line: int


def _line(page: str, offset: int) -> int:
    return page.count("\n", 0, offset) + 1


def _markers(page: str) -> list[_Marker]:
    """Every marker of `page` in order; a comment naming `progress:` that is no marker fails."""
    found = []
    end = 0
    for m in COMMENT.finditer(page):
        end = m.end()
        if MARKER_WORD not in m.group():
            continue
        line = _line(page, m.start())
        marker = MARKER.fullmatch(m.group())
        closing = marker is not None and marker.group(1) == "/"
        if marker is None or closing == (marker.group(3) is not None):
            msg = f"line {line}: malformed progress marker {m.group()[:80]!r}"
            raise ProgressReportError(msg)
        found.append(_Marker(closing, marker.group(2), marker.group(3), m.start(), m.end(), line))
    if (rest := page.find("<!--", end)) != -1:
        msg = f"line {_line(page, rest)}: unterminated HTML comment"
        raise ProgressReportError(msg)
    return found


def parse(page: str, generated: Collection[str], judgment: Collection[str]) -> dict[str, Block]:
    """The blocks of an existing page by name; `generated` and `judgment` are the block names
    known of each kind (the page layout, page.LAYOUT).

    Fails (ProgressReportError) on an unknown or duplicate name, a marker inside a judgment block,
    nested or unbalanced markers, and malformed or unterminated marker comments.
    """
    known = {GENERATED: generated, JUDGMENT: judgment}
    blocks: dict[str, Block] = {}
    current: _Marker | None = None
    for m in _markers(page):
        if current is not None and current.kind == JUDGMENT and not (m.closing and m.kind == JUDGMENT):
            msg = (
                f"line {m.line}: judgment block {current.name!r} (opened at line {current.line}) "
                "contains a progress marker; judgment text may not hold one"
            )
            raise ProgressReportError(msg)
        if not m.closing:
            if current is not None:
                msg = f"line {m.line}: nested marker inside block {current.name!r} (line {current.line})"
                raise ProgressReportError(msg)
            if m.name not in known[m.kind]:
                msg = f"line {m.line}: unknown {m.kind} block name {m.name!r}"
                raise ProgressReportError(msg)
            if m.name in blocks:
                msg = f"line {m.line}: block {m.name!r} appears twice"
                raise ProgressReportError(msg)
            current = m
            continue
        if current is None:
            msg = f"line {m.line}: unbalanced marker: closes a {m.kind} block that is not open"
            raise ProgressReportError(msg)
        if current.kind != m.kind or current.name is None:
            msg = (
                f"line {m.line}: unbalanced marker: closes a {m.kind} block, but the open block "
                f"{current.name!r} (line {current.line}) is {current.kind}"
            )
            raise ProgressReportError(msg)
        blocks[current.name] = Block(
            current.kind, current.name, page[current.end : m.start], current.end, m.start
        )
        current = None
    if current is not None:
        msg = f"line {current.line}: unbalanced marker: block {current.name!r} is never closed"
        raise ProgressReportError(msg)
    return blocks


def as_of(text: str) -> tuple[str, str] | None:
    """The (date, commit) of a judgment block's as-of line, its first non-blank line; else None."""
    first = next((line.strip() for line in text.splitlines() if line.strip()), "")
    m = AS_OF.fullmatch(first)
    return (m.group(1), m.group(2)) if m else None


def stale_line(text: str | None, run: Run) -> str | None:
    """The generated line under a judgment block's heading when its as-of does not name `run`.

    None for a block that is missing or still the placeholder, and for one written for this run.
    """
    if text is None or text == PLACEHOLDER:
        return None
    written = as_of(text)
    if written is None:
        return "_Written for an unnamed run (no as-of line); not yet updated for this run._"
    if written == (run.date, run.commit):
        return None
    return f"_Written for the run of {written[0]}; not yet updated for this run._"


def judgment(name: str, heading: str, old: dict[str, Block], run: Run | None) -> str:
    """Judgment block `name` for the new page: its `heading`, any stale line, and the old text
    between its markers byte for byte (the placeholder when the old page has no such block).
    With no `run` (a page with no complete run yet), no stale line is written."""
    block = old.get(name)
    text = block.text if block is not None else None
    parts = [f"## {heading}"]
    if run is not None and (line := stale_line(text, run)) is not None:
        parts.append(line)
    parts.append(
        open_marker(JUDGMENT, name) + (PLACEHOLDER if text is None else text) + close_marker(JUDGMENT)
    )
    return "\n\n".join(parts)


def generated(name: str, body: str) -> str:
    """Generated block `name` holding `body`, each marker on its own line."""
    return f"{open_marker(GENERATED, name)}\n{body.strip(chr(10))}\n{close_marker(GENERATED)}"


def replace_generated(page: str, block: Block, body: str) -> str:
    """`page` with generated `block` (from `parse(page, ...)`) holding `body` instead, laid out as
    `generated` writes it; every byte outside the block's markers is kept."""
    return page[: block.start] + f"\n{body.strip(chr(10))}\n" + page[block.end :]
