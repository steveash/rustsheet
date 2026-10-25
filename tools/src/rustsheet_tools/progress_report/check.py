"""`rustsheet-progress-report --check`: the judgment blocks of docs/progress.md (SPEC-0022, T-0074).

A judgment block passes when its as-of line names the run in the `status` block, every number in
it appears verbatim in a generated block, and it is plain text: no URL, no Markdown link, image or
reference definition, and no `<` tag apart from its own as-of comment. The page is parsed by
`page.parse` (blocks.py's parser, with page.py's `LAYOUT` names), so the check sees exactly the
blocks the generator sees.
"""

from __future__ import annotations

import re

from rustsheet_tools.progress_report import blocks, page
from rustsheet_tools.progress_report.model import COMMIT_SHA, RUN_DATE, ProgressReportError

OK = "progress check ok"
# The run the `status` block names; page.status writes this sentence (a test keeps the two in step).
STATUS_RUN = re.compile(rf"The eval run of \*\*({RUN_DATE.pattern})\*\* at commit `({COMMIT_SHA.pattern})`")
# What counts as a number (T-0074). A trailing `,` is the sentence's, not the number's.
NUMBER = re.compile(r"\d[\d,]*(?:\.\d+)?%?")
# Digit runs that are names, not measurements: the as-of comment, dates, task, spec, PR and
# decision ids, milestones.
IGNORED = re.compile(
    rf"{blocks.AS_OF.pattern}|{RUN_DATE.pattern}"
    r"|\bT-\d{4}\b|\bSPEC-\d{4}\b|#\d+|\b[Dd]ecision \d{4}\b|\bM\d+\b"
)
# Plain text only (SPEC-0022 "Plain text only"): each pattern names what it rejects.
NOT_PLAIN = (
    ("a URL", re.compile(r"https?://", re.IGNORECASE)),
    ("a Markdown image", re.compile(r"!\[[^\]]*\]\([^)]*\)")),
    ("a Markdown link", re.compile(r"(?<!!)\[[^\]]*\]\([^)]*\)|\[[^\]]*\]\[[^\]]*\]")),
    ("a reference definition", re.compile(r"^ {0,3}\[[^\]]+\]:", re.MULTILINE)),
    ("a tag", re.compile(r"<[A-Za-z/!]")),
)


def numbers(text: str) -> list[str]:
    """The numbers of `text` in order, as printed, after dropping the ignored digit runs."""
    return [m.group().rstrip(",") for m in NUMBER.finditer(IGNORED.sub(" ", text))]


def _body(text: str) -> str:
    """A judgment block's text without its as-of line (the first non-blank line, when it is one)."""
    if blocks.as_of(text) is None:
        return text
    lines = text.splitlines(keepends=True)
    first = next((i for i, line in enumerate(lines) if line.strip()), 0)
    return "".join(lines[:first] + lines[first + 1 :])


def offenders(text: str) -> list[str]:
    """Every reason the judgment blocks of page `text` fail the check; empty when they pass.

    Fails (ProgressReportError) when the page's markers do not parse or its `status` block names
    no run.
    """
    parsed = page.parse(text)
    status = parsed.get("status")
    run = STATUS_RUN.search(status.text) if status is not None else None
    if run is None:
        msg = "the `status` block names no run date and commit, so no as-of line can match it"
        raise ProgressReportError(msg)
    cited = {n for b in parsed.values() if b.kind == blocks.GENERATED for n in numbers(b.text)}
    found = []
    for name in page.JUDGMENT_BLOCKS:
        block = parsed.get(name)
        if block is None:
            found.append(f"{name}: the judgment block is missing")
            continue
        written = blocks.as_of(block.text)
        if written is None:
            found.append(f"{name}: no as-of line (its first line must be `<!-- as-of: <date> <sha> -->`)")
        elif written != (run.group(1), run.group(2)):
            found.append(
                f"{name}: as-of {written[0]} {written[1]} does not name the run in `status`, "
                f"{run.group(1)} {run.group(2)}"
            )
        body = _body(block.text)
        found += [
            f"{name}: holds {what}, {body[m.start() :].partition(chr(10))[0][:60]!r}; "
            "judgment blocks are plain text only"
            for what, pattern in NOT_PLAIN
            for m in pattern.finditer(body)
        ]
        found += [
            f"{name}: the number {n} appears in no generated block"
            for n in dict.fromkeys(numbers(body))
            if n not in cited
        ]
    return found
