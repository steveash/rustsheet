---
id: T-3052
title: "spec2xlsx keeps a formula's trailing } (XlsxWriter strips it and corrupts the workbook)"
milestone: M2
lane: harness
status: todo
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
- [ ] A tools unit test builds a case whose dynamic and legacy formulas end with `}` and reads the `<f>` text back
      unchanged.
- [ ] Every existing probe case's xlsx is byte-identical except for formulas that ended with `}` (none should exist
      in committed cases, since they could not be filled).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change.
