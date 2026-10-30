---
id: T-2860
title: "CHISQ.INV, CHISQ.INV.RT and CHIINV at df 2 and GAMMA.INV at alpha 1 are Excel's closed forms -2*LOG1P(-p), -2*LN(p) and -beta*LOG1P(-p) (34 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-0972]
campaign: accuracy
---

## Goal
With a deg_freedom that truncates to 2, CHISQ.INV, CHISQ.INV.RT and CHIINV, and with alpha exactly 1, GAMMA.INV and
GAMMAINV, return Excel's doubles: the exponential distribution's closed forms instead of a search on the incomplete
gamma.

## Context
Gap triage 2026-10-27 (opus refill). Rule: docs/excel-semantics/statistics.md "Chi-squared quantile at df 2 and
gamma quantile at alpha 1 are closed forms (T-2860)":
- `CHISQ.INV(p, 2)` = `-2 * ln_1p(-p)`;
- `CHISQ.INV.RT(q, 2)` = `CHIINV(q, 2)` = `-2 * ln(q)` (the upper-tail p itself, never `1 - q`);
- `GAMMA.INV(p, 1, beta)` = `GAMMAINV(p, 1, beta)` = `-beta * ln_1p(-p)`.
Evaluated with Rust's `f64::ln` and `f64::ln_1p` in that operation order, all 34 failing cells of these shapes in
eval/ match Excel bit for bit (the triage checked them in Python, whose `math.log`/`math.log1p` are the same libm
calls). If the engine routes LN through an Excel-specific kernel elsewhere (numbers.md, probes/ln-fyl2x-kernel), keep
plain libm here unless a cell disagrees, and record which one matched.
Examples: probes-numeric/m9-legacy-chiinv A23 `CHIINV(0.01,2)`: Excel 9.210340371976182, ours 9.210340371976189;
m9-gamma-inv A20 `GAMMA.INV(0.999,1,1)`: Excel 6.907755278982136, ours 6.907755278982083; gen-m9/8fec7b207f4dd753
A265 `CHISQ.INV.RT(0.001,2)`: Excel 13.815510557964274, ours 13.815510557964235; t1543-inverse-cdf-tiny-p A55
`GAMMA.INV(1E-300,1,1)`: Excel 1E-300, ours 9.999999999999999E-301. The forms are the accurate ones (Excel is close to
correctly rounded and the search is not), hence `accuracy`; 4 cells are visibly off, the rest 1 to 40 ulp.

T-0972 (done) reworked `gamma_quantile` (functions/statistical/incomplete_gamma/mod.rs) on main; add the
closed-form branch on top of it, before that search, in `chisq_inv_of` (functions/statistical/chisq.rs) and the GAMMA.INV caller. Argument
checks and errors stay as they are (p = 0, p = 1, df out of range).

Cells (2026-10-27; 12 lines in eval/baseline/gen-m9.txt, 22 in the ungated probes-numeric suite):
- gen-m9/3e98b26cf6713e77.case.json (1): A326
- gen-m9/4ab2ba98d26c8a5e.case.json (1): A315
- gen-m9/8d3588d8df0812df.case.json (3): A130, A289, A293
- gen-m9/8f2ac25945af87da.case.json (1): A301
- gen-m9/8fec7b207f4dd753.case.json (1): A265
- gen-m9/984cba67bbe6972b.case.json (1): A286
- gen-m9/a612e89321a6df9c.case.json (1): A307
- gen-m9/abc2939ebd1e2365.case.json (1): A337
- gen-m9/c2974c061384fdd9.case.json (1): A297
- gen-m9/ecc87f80077959e0.case.json (1): A154
- gen-m9/ee2497a5e683c5d9.case.json (1): A56
- gen-m9/f34f66fb7f81e673.case.json (1): A102
- probes-numeric/m9-chisq-inv-rt.case.json (2): A9, A10
- probes-numeric/m9-chisq-inv.case.json (2): A12, A13
- probes-numeric/m9-gamma-inv.case.json (4): A19, A20, A26, A27
- probes-numeric/m9-legacy-chiinv.case.json (6): A13, A14, A23, A24, A33, A34
- probes-numeric/t1049-chisqinv.case.json (2): A14, A18
- probes-numeric/t1049-chisqinvrt.case.json (2): A2, A6
- probes-numeric/t1543-inverse-cdf-tiny-p.case.json (2): A55, A58

## Acceptance criteria
- [ ] The 12 gen-m9 lines above disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new
      known-failure line in any baseline.
- [ ] The 22 probes-numeric cells above pass in `make eval` (name any that do not in the Outcome).
- [ ] Unit tests for `CHIINV(0.01,2)` = 9.210340371976182, `CHISQ.INV(0.999,2)` = 13.815510557964272,
      `CHISQ.INV.RT(0.001,2.5)` = 13.815510557964274 and `GAMMA.INV(0.999,1,1000000)` = 6907755.2789821355.
- [ ] statistics.md's T-2860 entry gets an "Implemented in" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other df and alpha (T-0972, T-2561, T-2695). gen-m9 4ab2ba98d26c8a5e A315, 8d3588d8df0812df A130 and
  abc2939ebd1e2365 A337 are also in deferred T-2695's list: they are claimed here, and this task's PR drops them there; CHISQ.INV.RT at df 1 (statistics.md T-1634 entry).

## Visible-parity stage
Decisions 0083 and 0114, stage `accuracy`: done when every cell this task owns is within half a unit of the 15th
significant digit of Excel's value; the closed forms are expected to be bit-exact, and any ulp residue moves to a
`defer: phase-1b` follow-up named in the Outcome.

## Outcome
Already fixed on main by T-3016 (`exponential_quantile`, shape-1 closed form in the smaller tail) before this task
was picked: none of the 34 cells fails in `make eval` (the 12 gen-m9 lines were already absent from
eval/baseline/gen-m9.txt; all 22 probes-numeric cells pass). This PR adds the unit test
`t2860_df2_and_alpha1_cells_are_excels_doubles` (the four acceptance values, bit-exact) and the statistics.md
"Implemented in" line. No eval delta, no baseline change.
Also dropped the three cells it shared (4ab2ba98d26c8a5e A315, 8d3588d8df0812df A130, abc2939ebd1e2365 A337) from T-2695's list.
