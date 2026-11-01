---
id: T-3231
title: "F.DIST density last digits on t2513-r3-fdensity-sw and -holdsw after M-final (1,584 cells 1-8 ulp, all within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586, T-3173, T-3174]
defer: phase-1b
---

## Goal
`F.DIST(x,d1,d2,FALSE)` matches Excel to the bit on the F density cells listed below. Every one of them is already
within half a unit of Excel's 15th significant digit after T-3034's M-final port, so this is the Phase 1b follow-up of
T-3034 that its Out of scope ("Bit-exactness and the |P| >= 32 power-term noise") left unfiled.

## Context
- Gap triage 2026-10-30 (`make eval` at e1255db9, excel-graph evidence). No open task names these two cases: T-3034
  (done) owned their visibly-off cells and moved all of them within 15 digits. T-3173 (power term where |c·LN x| >= 4)
  and T-3174 (constant K where |c·LN x| < 4) own the F density's remaining visibly-off cells in other sweeps. T-2586
  (research, in progress) is cracking the density pieces M-final leaves open. Every failing cell of the two cases is
  listed here: 1,327 of 2,400 checked in t2513-r3-fdensity-sw and 255 of 400 in t2513-r3-fdensity-holdsw.
- Ulp distance, ours against Excel's, across the 1,584 cells: 1 ulp 1,245, 2 ulp 254, 3 ulp 53, 4 ulp 16,
  5 to 8 ulp 16. The verdicts are 1,568 `NumericUlp` and 16 `NumericClose`, and none is visibly off.
- Examples (`make eval-show CASE=probes-numeric/t2513-r3-fdensity-holdsw.case.json`), Excel against ours:
  - t2513-r3-fdensity-holdsw Calc!A1 `=F.DIST(1.6,5,1,FALSE)`: 0.13173352591233384 against 0.1317335259123338 (1 ulp).
  - t2513-r3-fdensity-holdsw Calc!A103 `=F.DIST(1.6,9,3,FALSE)`: 0.22227451779760707 against 0.22227451779760699 (3 ulp).
  - t2513-r3-fdensity-sw Calc!A1755 `=F.DIST(2.75,7,75,FALSE)`: 0.02899589729965933 against 0.02899589729965936 (8 ulp).
  - t2513-r3-fdensity-sw Calc!A1765 `=F.DIST(2.75,7,85,FALSE)`: 0.02795470315770414 against 0.02795470315770411 (8 ulp).
- Suspected root cause: M-final reproduces Excel's structure E = K + fl(c·LN x) - h·L(r), F = fl(EXP(E)/x), but its
  constant K (exact -lnB ± a·ln(m/n) plus a fitted delta table) and the rounding of the power term are not Excel's to
  the bit. docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)"
  and its T-2586 round 4 and 5 updates hold the rule. The model is
  docs/research/t2464/r5-f-final/fdensity_excel.py (RESULTS.md sections 4 and 6). This is numeric fitting, so the tier
  is opus. It waits on T-2586's bit-level model and on T-3173/T-3174, which change the same `excel_f_density` in
  crates/rustsheet-engine/src/functions/statistical/f_density.rs.
- Evidence: these two sweeps (2,800 cells) plus T-2586's round 4 and 5 sweeps are enough points to score candidate
  constants. If T-2586's model still cannot separate candidates on the 8-ulp `NumericClose` cells (d1 = 7, x = 2.75,
  d2 = 75 to 85), file a research-lane oracle sweep over that d2 band before fitting, and depend on it.

### Cases owned
- probes-numeric/t2513-r3-fdensity-sw: all 1,327 failing F.DIST density cells
- probes-numeric/t2513-r3-fdensity-holdsw: all 255 failing F.DIST density cells
- probes-numeric/t1049-fdist.case.json (1): Calc!A6
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A65 (its `cumulative` argument is the boolean
  `IFERROR(((-1)>1E15),..)`, which is FALSE, so it is a density)

## Acceptance criteria
- [ ] The known-failure line gen-m9/8fec7b207f4dd753.case.json Calc!A65 disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), and no baseline gains a line (`make eval-check`: 0 regressions).
- [ ] probes-numeric is ungated (no baseline file), so the owned probes-numeric cells are scored from `make eval`'s
      report.json: the 1,583 cells above pass bit-exact, and no other F.DIST cell of probes-numeric moves from passing
      to failing. Or, under the decision 0056 time box, the Outcome records each model tried with its bit-exact count
      on the owned cells and on T-2586's hold-out sets, and the cells still open move to a follow-up.
- [ ] The final form is recorded in docs/excel-semantics/statistics.md's F density entry, with these cases as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F density's visibly-off cells (T-3173, T-3174) and T-2586's research model itself.
- The three unlisted F cdf cells (`F.DIST(..,TRUE)`, 1-2 ulp) of the T-1049 F.DIST sweep. They go through the
  incomplete beta's cdf path (T-2221, T-2508, T-2510), not this density.
