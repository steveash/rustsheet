---
id: T-3243
title: "BETA.DIST(x,1025,1.5,FALSE): Excel's Loader density at a large exponent (t2464-r2-log1p-hix column C, 4,180 unowned visibly-off cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3242]
campaign: visible
---

## Goal
`BETA.DIST(x,a,b,FALSE)` at large shapes, where the Loader exponent is large, matches Excel's value to 15 digits.
That covers the 4,180 visibly-off cells of T-2464's a = 1025, b = 1.5 column and T-3242's sweep, with no new
visibly-off cell anywhere.

## Context
- Gap triage 2026-10-30c (`make eval` on main 40ca52179, excel-graph; `python3 scripts/exit_owners.py --all-off`,
  also run against the task files of every unmerged branch): no open task owned these cells. T-2507 (done) ported
  Excel's Loader saddle point (`crates/rustsheet-engine/src/functions/statistical/loader.rs`
  `excel_beta_density`). The column's 5,264 cells are at a + b - 2 = 1024.5. 5,231 of them still differ from Excel
  at the bit, and 4,180 are visibly off. Column B (b = 1) passes.
- Examples (Excel vs ours):
  - Calc!C1 `=BETA.DIST(A1,1025,1.5,FALSE)`, A1 `=(83886080*2^-27+31*2^-53)`: 2.4480589855435217e-205 vs
    2.448058985543529e-205
  - Calc!C2000: 4.406866092326905e-114 vs 4.406866092326949e-114
  - Calc!C4000: 1.6760890714342508e-38 vs 1.6760890714342558e-38
  - Calc!C5000: 2.6526585654535142e-5 vs 2.65265856545351e-5
- Root cause (suspected): an error at the scale of the exponent. The sd of (ours - Excel) in units of 2^-52 relative
  grows with |ln f|: 14 at 0-50, 52 at 100-150, 110 at 250-350. It falls back to 18-38 above 400 (T-3242's Context
  has the table). That is the signature of one rounding at |E| scale that Excel makes and the engine does not, or the
  reverse. The model of record (docs/excel-semantics/statistics.md, "Beta density for min(a,b) > 1 is Loader's
  saddle point") holds E extended into EXP. Its stirlerr and bd0 pieces were fitted at n <= 256, with an n trend
  measured only to n = 105. The larger shapes may store something the fit never saw.
- Stage `visible`, not `accuracy`. On a 300-cell sample against 200-bit mpmath, Excel is a median 92 ulp from the
  correctly rounded density (p90 323), and the engine is 69 (p90 276). EXP of the double nearest the exact ln f is
  not Excel's value (91 ulp).
- docs/research/t2464/r2-log1p/RESULTS.md section 5: for x > 0.625 the density's ln x is Excel's LN(x), not
  alnrel(-y). It calls this column (b = 1.5) "inconclusive", with an unsolved composition.
- **Needs an oracle sweep (T-3242).** One shape cannot separate what depends on n, on k and on bd0's arguments, and
  the fit needs a hold-out no choice uses (decision 0056). Fit on this column and T-3242's fit case. Score once on
  T-3242's hold-out. Time-box per decision 0056 and record negatives in statistics.md.
- Coordinate with T-3244 (two large shapes, n 90-256) and T-2586 (research on the Loader constant and the bd0
  series). If one of them lands a piece first, start from it. T-3173 (the F density's power term) shows the same
  pattern of error growing with the exponent.

Cells (gap triage 2026-10-30c): every cell of this case:
- probes-numeric/t2464-r2-log1p-hix.case.json

## Acceptance criteria
- [ ] Every cell of the case, and every cell of T-3242's two cases, is within 15 digits of Excel's value, or each
      cell left over is named here with its reason and an owner.
- [ ] No new visibly-off cell in any suite: the cells of T-2507, T-3220, T-3244 and the min(a,b) <= 1 tasks keep
      their verdicts, and the known-failure lines in eval/baseline/probes-numeric.txt only shrink.
- [ ] statistics.md's Loader entry records the large-shape piece found, with its hold-out score (exact, 4 ulp and
      15 digits, and CR on the same rows), or the negatives tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- min(a,b) <= 1 (T-2503, T-2821, T-3035) and column B of the case.
- Both shapes large at n < 300 (T-3244), and the residue at n < 90 (T-3220).

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value. Bit-level
residue goes to a follow-up with `defer: phase-1b`.
