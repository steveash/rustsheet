---
id: T-0074
title: "Progress report --check: every number in a judgment block appears in the generated part of docs/progress.md"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-0070]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
`cd tools && uv run rustsheet-progress-report --check` lets a reviewer verify mechanically that
the steward's judgment text cites only numbers the page generated, for the current run.

## Context
- SPEC-0022, "Judgment blocks" ("Cited numbers").
- The marker parser is `progress_report/blocks.py` (T-0070). Reuse it; do not write a second
  parser.

## Acceptance criteria
- [x] Module `progress_report/check.py` reads docs/progress.md and exits 1, listing every
      offender, in three cases:
  - a judgment block's `as-of` line does not name the date and commit in the `status` block;
  - a number in a judgment block does not appear verbatim in a generated block;
  - a judgment block contains anything but plain text: an `http://` or `https://` URL, a
    Markdown link or image (`[..](..)`, `![..](..)`, `[..][..]` or a reference definition), or a
    `<` tag (`<` followed by a letter, `/` or `!`) other than its own as-of comment.
  Otherwise it prints `progress check ok` and exits 0.
- [x] What counts as a number: a token matching `\d[\d,]*(\.\d+)?%?`. The check ignores:
  - `YYYY-MM-DD` dates;
  - `T-\d{4}`, `SPEC-\d{4}`, `#\d+` and `decision \d{4}`;
  - milestone names `M\d+`;
  - text inside the as-of comment.
- [x] Tests cover a cited number, an uncited one, a percentage, a stale as-of, each ignored
      form, and each plain-text violation (URL, link, image, reference definition, tag).
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- Judging whether the text is right (the skill, T-0075).
- A Makefile target: the skill calls the entry point directly.

## Outcome
`rustsheet-progress-report --check` (new `progress_report/check.py`, a small `--check` hook in
`__init__.py`; `--out-dir` is now required only without `--check`, which defaults to the repo's
docs directory). It parses the page with `page.parse` (blocks.py's parser; no second one) and
exits 1 listing every offender: a missing judgment block, a missing or stale as-of line against
the run named in the `status` block (an "unknown commit" status fails on purpose), any number not
printed in a generated block, and any text that is not plain.
- **Numbers** are compared as exact tokens, so `1234` does not match `1,234`, and a trailing comma
  is dropped. Digits inside identifiers (LOG10, ATAN2, v15, phase1, M2) are not numbers on either
  side. Ignored digit runs are the as-of comment, dates, `T-`/`SPEC-` ids, `decision nnnn` and
  `#n`. `#n` is kept because risks cite PR numbers. No separate milestone rule is needed: `M2` is
  an identifier.
- **Plain text** is stricter than the spec's list, after security review. Any `[` or `]` fails,
  which covers every link, image and reference-definition form, including nested, escaped and
  indented ones. Any `<` other than the block's own as-of comment fails. URLs (`http(s)://`),
  `www.` addresses, email addresses, and the schemes ftp, mailto, xmpp, javascript, data, vbscript
  and file also fail. A scheme counts only when text follows its `:` directly, so prose such as
  "the data: 12 cells" passes. Each rule is listed once per block, with its first hit.

The generated block names come from page.py's `LAYOUT`, so blocks added later count as cited
sources with no change here. A page holding a placeholder fails, since it has no as-of line.
Tests: tools/tests/test_progress_check.py. No eval delta (tools only).
