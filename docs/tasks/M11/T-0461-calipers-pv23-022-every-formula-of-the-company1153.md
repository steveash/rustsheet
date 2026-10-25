---
id: T-0461
title: "Calipers pv23_022: every formula of the Company1153 3-Statement sheet computes 0 (308 cells)"
milestone: M11
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
The `Company1153 3-Statement` sheet of the calipers model pv23_022 computes Excel's values instead
of 0 in every formula cell.

## Context
Found by the Phase 1 assessment of 2026-10-25 (decision 0114); the largest visibly-off group in the
real-world calipers corpus. `make eval` on 2c52346: 308 visibly-off cells in
`calipers/pv_goldens/pv23_022/golden.xlsx`, all on `Company1153 3-Statement`, all 0 where Excel has
numbers, for example:
- D32 `(D$51*INDEX('Company1153 Assumptions'!$D$88:$D$90,MATCH($D$5,'Company1153 Assumptions'!$C$88:$C$90,0)))`: Excel 66.54103645300171, ours 0;
- D34 `D29+D32`: Excel 30782.701036453007, ours 0;
- D51 `D$108`: Excel 97308.20729060035, ours 0; D108 is `D106+D107`;
- D102 `MAX(D94+D99,0)*INDEX(...)`: Excel 24924.621871801006, ours 0.

Even D29 `D20-D27` is 0, so the cause is upstream of the lookups. The workbook was saved by
Microsoft Macintosh Excel with `<calcPr calcMode="manual"/>` and the sheet looks like a circular
3-statement model (D51 reads D108, which feeds back into the income statement). Suspects: a cycle
resolved to 0 without `iterate` (decision on cycles: docs/excel-semantics/recalc.md, T-0447), or a
manual-calc load path. Use `make eval-show CASE=calipers/pv_goldens/pv23_022/golden.xlsx`
(fetch first: `target/release/rustsheet-eval fetch --suite calipers`).

## Acceptance criteria
- [ ] The root cause is named in the Outcome and, if it is an Excel rule, in docs/excel-semantics/.
- [ ] The 308 cells are not visibly off; `rustsheet-eval check --suite calipers` shows no regression.
- [ ] A small spec case or unit test reproduces the cause without the calipers workbook.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- Other calipers models.
