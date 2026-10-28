---
id: T-0461
title: "Calipers pv23_022: every formula of the Company1153 3-Statement sheet computes 0 (308 cells)"
milestone: M11
lane: engine
status: done
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
- [x] The root cause is named in the Outcome and, if it is an Excel rule, in docs/excel-semantics/.
- [x] The 308 cells are not visibly off; `rustsheet-eval check --suite calipers` shows no regression.
- [x] A small spec case or unit test reproduces the cause without the calipers workbook.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- Other calipers models.

## Outcome
Root cause, an Excel rule (recalc.md, "A circular reference without iteration keeps the cells' values from before the
recalculation"). With iterative calculation off, a cell on a cycle, and every cell whose evaluation reads one, keeps the
value it had before the recalculation: its cached value in a loaded workbook, 0 for a formula never calculated. We
stored 0. The T-0445 probes that led to "is 0" were built by `rustsheet-spec2xlsx`, which caches every formula as 0, so
they could not tell "0" from "keeps 0".

Evidence:
- pv23_022 has two 3-statement loops (`Company1153` and `Company1808 3-Statement`, 7 columns x 11 cells each) and no
  `iterate`. An excel-graph recalculation of the file returns the cached loop values.
- New fixture fixtures/circular-keeps-cached-values (excel-graph): loop members and loop readers keep cached 5, 4, 999
  and 7, while F1 `=1+1` recalculates from a stale 55 to 2. A reader that also reads a changed cell keeps its cached
  value too. A formula whose evaluated branch does not read the loop recalculates.

Change: `Model::circular_value` (circular.rs). A circular plain formula, or a one-cell array anchor (pv23_022 D88
`=D37` is `cm="1"`), keeps its stored value; one never evaluated is 0. A spilling anchor on a loop stays 0 (no
evidence). Unit tests are in test_circular_references.rs.

Eval:
- calipers pv23_022 passes 1010/1010 (was 702).
- `check --suite calipers`: 908 fixed against 600 for a binary built from `main` 583620cc. The difference is the 308
  cells of pv23_022.
- The 36 regressions are the same on `main` (the calipers baseline is stale; it is not gated). Filed T-0463; the
  calipers baseline is not ratcheted here.
- Gated suites: 0 regressions; fixtures +1 case.

Follow-up from review: T-0464 probes the cases still at 0 with no Excel evidence (a loop member with no cached value, a
spilling anchor on a loop, a cached text, boolean or error on a loop).
