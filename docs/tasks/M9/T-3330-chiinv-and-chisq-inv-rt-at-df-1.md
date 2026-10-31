---
id: T-3330
title: "CHIINV and CHISQ.INV.RT at df 1 for tiny p: Excel lands one double-cdf band below the crossing of 1 - p (CHIINV(1E-9,1) 2.2E-7 off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3329]
campaign: visible
---

## Goal
`CHIINV(p,1)` and `CHISQ.INV.RT(p,1)` return Excel's quantile, within half a unit of its 15th significant digit,
for every p at which the engine reads the right tail as the lower quantile of 1 - p. For tiny p, Excel's search on its
double-valued cdf lands a whole band of flat cdf away from the crossing.

## Context
Gap triage 2026-10-31 (`make eval` on 1171772e, `python3 scripts/exit_owners.py --all-off`, excel-graph). No open
task named these cells. T-2561 (done) recorded the quirk in docs/excel-semantics/statistics.md ("CHISQ.INV.RT and
CHIINV quirks", the df-1 bullet, and its "Leads" list) but did not implement it.
- probes-numeric/m9-legacy-chiinv.case.json (2): Calc!A1, A2

`CHIINV(1e-09,1)` and `CHISQ.INV.RT(1e-09,1)`: Excel 37.324892889999234, ours 37.324893106518715 (6E-9 relative).
A 50-digit mpmath check (gap triage):
- the true quantile is 37.32489305136233;
- ours is the correctly rounded crossing of the double 1 - 1E-9 = 0.99999999900000002828 on the lower cdf, and
  erf(sqrt(x/2)) at ours rounds to that double;
- at Excel's it rounds to 0.9999999989999999, one ulp lower;
- the distance between them, 2.16519E-7, is exactly 2^-53 over the density 5.128E-10: one band of x whose double cdf
  is constant.

So Excel stops in the band below the one that holds 1 - p. That is the edge or start a bracketing or Newton search
on a double cdf would reach. Where it starts, which edge it takes and whether it does the same for every p is not in
eval/. `CHISQ.INV.RT(1E-10,1)` (41.82145620298276) passes, so the rule is not "always one band below". Needs an
oracle sweep: T-3329 adds Excel's quantiles for p = 1E-4 to 1E-16 and Excel's cdf at the bands beside four of them.

Related work. T-3146 (time-boxed) fits Excel's landing in the flat band near p = 0.999 at other shapes and records its
candidates in statistics.md. Its forms are the first to try here: the band is 1E-8 relative, not 1E-13, so
the signal is far clearer. Code: crates/rustsheet-engine/src/functions/statistical/chisq.rs, the `is_one_degree`
branch of the right-tail quantile (`gamma_quantile(.., 1.0 - p, Tail::Lower)`), and
incomplete_gamma.rs `gamma_quantile`.

## Acceptance criteria
- [ ] m9-legacy-chiinv Calc!A1 and A2 are within half a unit of Excel's 15th significant digit, and so is every
      df-1 and df-1.5 quantile cell of T-3329's case. If the rule cannot reach some, the Outcome lists them with
      the forms tried and their bit-exact and visible counts (decision 0056: two attempts, then time-box).
- [ ] No cell that passes today gets worse: `CHISQ.INV.RT(1E-10,1)`, the `#NUM!` at p of 2^-54 or less, and the df-2
      closed form. No new known-failure line in any eval/baseline file (`make eval-baseline`).
- [ ] The landing rule, with the T-3329 cells that pin it down, is in statistics.md's df-1 bullet, replacing "Excel
      searches its own double-valued cdf".
- [ ] Iteration stays bounded on untrusted input (the shared iteration budget; p next to 0 and next to 2^-54).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114: the two owned cells are visibly off, so this task is in the campaign. Excel is the inaccurate side
(1.6E-7 below the true quantile; ours is 5.5E-8 above it, the correctly rounded crossing of the double 1 - p), so the
stage is `visible`: reproduce Excel's search, never a more accurate kernel. Its target is that every owned cell is
within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up.

## Out of scope
- The p = 0.999 band at other df (T-3146) and the deep right-tail `#NUM!` edge at df 3 and above (T-3310).
- df 2 and above at tiny p (the upper-tail path, not 1 - p).
