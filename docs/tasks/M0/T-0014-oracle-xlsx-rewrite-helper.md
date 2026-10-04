---
id: T-0014
title: "Oracle tooling: one xlsx part-rewrite helper; rels-based formula-part scan"
milestone: M0
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-0013]
---

## Goal
One definition for "rewrite parts of an xlsx package" in tools/oracle, and a canary refusal scan
that cannot miss worksheets stored at nonstandard paths.

## Context
Review follow-ups from T-0013:
- `force_full_calc_on_load` (oracle/backends.py) and `embed_canary` (oracle/canary.py) both
  unzip, patch and rezip a package; the former hardcodes `xl/workbook.xml`, the latter resolves
  the workbook part from `_rels/.rels`. Share one helper and one workbook-part lookup.
- The `SHEETS()`/`INFO()` refusal scan in `embed_canary` covers the workbook part and
  `<wb_dir>/worksheets/`; resolve worksheet/macro-sheet targets from the workbook rels instead
  (also XLM `GET.WORKBOOK` in defined names).
- Optional: refuse packages whose total uncompressed size exceeds a cap before reading parts.

## Acceptance criteria
- [ ] A single helper rewrites package parts; both callers use it and the same workbook lookup.
- [ ] Refusal scan follows rels; tests cover a worksheet at a nonstandard target.
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing canary contents or the Graph session flow.
