---
id: T-2361
title: "Fit Excel's F.TEST kernel: bit-exact two-tailed p (not 2*MIN of its F.DIST)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
F.TEST/FTEST match Excel to the last digit. At minimum, the visibly-off cells T-1927 carried reach
15-digit parity without losing any cell that is bit-exact today.

## Context
Moved from T-1927 (its visible-parity criterion was not met). Visibly-off cells (gap triage
2026-10-21, decision 0083): probes-numeric/m9-legacy-ftest Calc!A11 `FTEST(B1:C5,B1:B5)` (Excel
0.9873866645220285), probes/stat-argument-modes Sheet1!H249/H250 (0.7947858780819458) and
H251 `FTEST(C1:C6,B1:B8)` (0.9195979047175995). The F tails are `f_cdf`/`f_sf` in
crates/rustsheet-engine/src/functions/statistical/fisher.rs (`fn_f_test`).
Findings so far (docs/excel-semantics/statistics.md "F.TEST / FTEST is not correctly rounded"):
- Excel's F.TEST is not `2*MIN` of its own F.DIST/F.DIST.RT (probes/ftest-decomposition A7/A8,
  B7/B8), and it is exactly symmetric in its two arguments.
- Over the 33 distinct inputs on file, Excel is -6..+4 ulps from the exact p, on both sides. Exact
  p is bit-exact on 12, the A&S 26.6.4-26.6.8 finite series in doubles on 11, the engine today on
  fewer, and none of two-pass/textbook/corrected variance or f = larger/smaller fits all.
  ironcalc-calc STATISTICAL/F_TEST.xlsx F1 (df 3,3) is bit-exact today and the exact p loses it.
- The T-1927 handoff tried an exact half-integer recurrence (1 ulp off on df 7,7 and 9,4).
The F.TEST, F.DIST and BETA.DIST rows of probes/ftest-decomposition in eval/baseline/probes.txt
are this task's (its DEVSQ rows are T-1749's).
Next: use the fitlab (tools, `kernels/beta.py`), with the 33 inputs as a case set. Candidates:
Excel's BETA.DIST kernel run on the reflected side; a series in
x = d2/(d2+d1 f) summed in reverse; long-double (x87) intermediates (decision on "Division rounds
twice"). Then a probe sweep over df 1..10 x a few f to separate the hypotheses.

## Acceptance criteria
- [ ] Visible parity: the 4 cells above have Excel's value to 15 significant digits, with no new
  known-failure lines in any baseline (`make eval-baseline` accepts).
- [ ] Either a bit-exact F.TEST kernel (H248/H251 exact), or the fit result written in
  docs/excel-semantics/statistics.md with the best candidate's score.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
