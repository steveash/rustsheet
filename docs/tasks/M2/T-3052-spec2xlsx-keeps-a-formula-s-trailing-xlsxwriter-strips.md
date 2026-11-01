---
id: T-3052
title: "spec2xlsx keeps a formula's trailing } (XlsxWriter strips it and corrupts the workbook)"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A case formula that ends with an array constant (`={0.1,1}+{0.2,1}-{0.3,2}`, `=0+{1;2}`) reaches Excel intact, so
the oracle can evaluate it.

## Context
- XlsxWriter's `write_array_formula`/`write_dynamic_array_formula` drop a trailing `}` (they strip the `{=...}`
  CSE wrapper), so tools/src/rustsheet_tools/spec2xlsx.py writes `<f>0+{0.1,1}+{0.2,1}-{0.3,2</f>` and the Graph
  `createSession` fails with 501 `FileCorruptTryRepair` / `unsupportedWorkbook` for the whole case. Found in T-3047,
  which worked around it by writing `-({0.3,2})`. Plain (legacy) `<f>` formulas lose the brace as well.
- Fix in spec2xlsx (e.g. write the formula so XlsxWriter's strip cannot apply, or patch the `<f>` text afterwards),
  not in the case files.

## Acceptance criteria
- [x] A tools unit test builds a case whose dynamic and legacy formulas end with `}` and reads the `<f>` text back
      unchanged.
- [x] Every existing probe case's xlsx is byte-identical except for formulas that ended with `}` (none should exist
      in committed cases, since they could not be filled).
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change.

## Outcome
Already fixed by T-3051 (`_cell_formula` in spec2xlsx.py re-appends the `}` XlsxWriter strips; covered by
`tools/tests/test_spec2xlsx_trailing_brace.py`, dynamic, legacy and bare-name cells). Verified the test passes on
current `origin/main`; no code change. Eval delta: none.
Byte-identity (criterion 2): `_cell_formula` changes only text ending in `}`; the only committed strings of that
shape are `={1,2,3}` in the 21 synthetic `eval/suites/gen-m8a` cases (not oracle-filled), so no probe case's xlsx moves.
