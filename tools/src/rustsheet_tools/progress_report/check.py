"""`rustsheet-progress-report --check`: the judgment blocks of docs/progress.md (SPEC-0022, T-0074).

A judgment block passes when its as-of line names the run in the `status` block (the complete
run, or the incomplete run a Stale banner reports), every number in
it appears verbatim in a generated block, and it is plain text: no URL, web or email address, no
`[` or `]` (so no Markdown link, image or reference definition in any form), and no `<` apart from
its own as-of comment. The page is parsed by
`page.parse` (blocks.py's parser, with page.py's `LAYOUT` names), so the check sees exactly the
blocks the generator sees.
"""

from __future__ import annotations

import re

from rustsheet_tools.progress_report import blocks, page
from rustsheet_tools.progress_report.model import COMMIT_SHA, RUN_DATE, ProgressReportError

OK = "progress check ok"
# The run the `status` block names: page.status writes "The eval run of ..." on a complete run and
# page.stale_status's banner "The run of ..." on a stale one (tests keep both in step). The banner's
# "last complete run, **<date>** at ..." never matches, so a stale page's as-of lines must name the
# incomplete run the banner reports, not the run its numbers come from.
STATUS_RUN = re.compile(
    rf"The (?:eval )?run of \*\*({RUN_DATE.pattern})\*\* at commit `({COMMIT_SHA.pattern})`"
)
# What counts as a number (T-0074): `\d[\d,]*(\.\d+)?%?` standing alone, never the digits of an
# identifier such as LOG10, ATAN2, v15 or phase1 (no word character directly before). A unit suffix
# does not hide a number: `500x`, `12ms` and `1.5x` still yield 500, 12 and 1.5, so a judgment block
# cannot state an uncited figure that way. A trailing `,` is the sentence's, not the number's.
NUMBER = re.compile(r"(?<!\w)\d[\d,]*(?:\.\d+)?%?(?![.,]\d)")
# Digit runs that are names, not measurements: the as-of comment, dates, task, spec, PR and
# decision ids. Milestone names (M2) need no entry: NUMBER never matches digits after a letter.
IGNORED = re.compile(
    rf"{blocks.AS_OF.pattern}|{RUN_DATE.pattern}"
    r"|\bT-\d{4}\b|\bSPEC-\d{4}\b|#\d+|\b[Dd]ecision \d{4}\b"
)
# Plain text only (SPEC-0022 "Plain text only"): (what is wrong, pattern). Banning every bracket and
# every `<` outright closes the nested, escaped and indented link and reference-definition forms and
# every tag form that a narrower pattern would miss. A scheme counts only when text follows its `:`
# directly, so prose such as "the data: 12 cells" stays plain text.
NOT_PLAIN = (
    ("a URL", re.compile(r"https?://", re.IGNORECASE)),
    ("a web address", re.compile(r"\bwww\.", re.IGNORECASE)),
    (
        "a URL scheme",
        re.compile(r"\b(?:ftp|mailto|xmpp|javascript|data|vbscript|file):(?=\S)", re.IGNORECASE),
    ),
    ("an email address", re.compile(r"\S@\S+\.\w")),
    ("square brackets are not allowed (links)", re.compile(r"[\[\]]")),
    ("`<` is not allowed (tags)", re.compile(r"<")),
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
    # An "unknown commit" status fails on purpose: an as-of line needs a sha to name the run.
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
        for what, pattern in NOT_PLAIN:
            hits = list(pattern.finditer(body))
            if hits:
                first = body[hits[0].start() :].partition("\n")[0][:60]
                more = f" (and {len(hits) - 1} more)" if len(hits) > 1 else ""
                found.append(f"{name}: {what}: {first!r}{more}; judgment blocks are plain text only")
        found += [
            f"{name}: the number {n} appears in no generated block"
            for n in dict.fromkeys(numbers(body))
            if n not in cited
        ]
    return found
