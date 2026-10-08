---
id: T-1013
title: "IMAGE, STOCKHISTORY, FIELDVALUE return their offline errors (SPEC-0003 F1)"
milestone: M1
lane: functions-logic
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1010, T-1012]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
IMAGE, STOCKHISTORY and FIELDVALUE evaluate to Excel's offline errors (`#BLOCKED!`, `#CONNECT!`, `#FIELD!`) instead of `#NAME?`, with no I/O.

## Context
SPEC-0003 "Functions with no I/O" (rollout F1). Known failures: probes/rich-errors-image (5 cells), -stockhistory (4) and -field (4). Argument-error precedence comes from T-1012's probe.

## Acceptance criteria
- [ ] The three functions are registered with Excel's arity (including the `_xlfn.` prefix). They return the spec's error after argument errors as T-1012 found, never spill, and do no I/O.
- [ ] information.md entries for IMAGE and STOCKHISTORY have `Status: DEVIATION (decision 0014)`: reachable URLs and real tickers give errors, where Excel fetches. FIELDVALUE's entry is VERIFIED.
- [ ] COUNTIF-family criteria naming the new errors follow T-1012's evidence. `is_english_error_string` changes only if Excel matches the error, and the test cites the probe cell.
- [ ] The 13 rich-errors-* known failures pass and are recorded with `make eval-baseline`, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Linked data types, image values, and stock data (decision 0027).

## Handoff (resumed)
Implemented and working on the branch: functions registered (`functions/information/offline_errors.rs`,
enum/lookup/`to_xlsx_string`/dispatch in `functions/mod.rs`, signatures in `static_analysis.rs`), names added to
`tools/upstream/generate_language/src/languages.json` + `main.rs` + `language/mod.rs` and `language.bin`
regenerated (non-English names are the English ones: no evidence for localized spellings), unit tests in
`test/test_fn_offline_errors.rs`, information.md entries (IMAGE/STOCKHISTORY DEVIATION, FIELDVALUE VERIFIED).
Eval: probes known failures 255 -> 218, 0 regressions; rich-errors-* all pass; `make eval-baseline` done.

Blocked: `make check` fails only the file-size ratchet (`scripts/check_file_size.py`):
`functions/mod.rs` grew 2874 -> 2887 and `parser/static_analysis.rs` 2157 -> 2171 ("oversized inherited
files may only shrink"). Registering any function grows both. I did not run `--update` (that would raise
the lock, which CLAUDE.md directive 2 forbids). Next: shrink those two files by at least the growth (e.g. merge
the ~88 runs of adjacent identical arms in `static_analysis.rs`, or split the per-function tables out of
`functions/mod.rs`), then `python3 scripts/check_file_size.py --update`, re-run `make check`, then the
review skill and the PR. Remaining after that: tick the acceptance boxes and re-add the Outcome note
(eval numbers above).
