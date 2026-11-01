---
id: T-1811
title: "PERCENTILE.INC interpolation matches Excel to the last bit (PERCENTILE(C1:C6,0.3) is 1.9000000000000001)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
PERCENTILE and PERCENTILE.INC give Excel's bits where the interpolation rounds differently today.

## Context
probes/stat-argument-modes H424 `PERCENTILE(C1:C6,0.3)` and H429 `PERCENTILE.INC(C1:C6,0.3)` (C1:C6 = 1, "2", TRUE, "x", 4, blank, so the numbers are 1 and 4): Excel 1.9000000000000001, ours 1.9 (`percentile_inc_impl` in functions/statistical/percentile.rs computes `a + frac*(b-a)`; `(1-frac)*a + frac*b` is 1.9 too). Find the evaluation order Excel uses (more probe cells with two- and three-point data and several k, excel-probe skill) before changing the formula; QUARTILE.INC shares it. Named in T-1772's Outcome.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): PERCENTILE and PERCENTILE.INC interpolation off Excel's, including tiny k where Excel's formula returns the lower value exactly (`PERCENTILE.INC(Data!B1:D6,1E-20)` Excel 0, ours 1.1E-20; `PERCENTILE.INC(Data!B8:E9,1E-16)` 4.4408920985006264E-17 against 5.0000000000000005E-17), which constrains the formulation.

Gap triage 2026-10-25: a formulation that reproduces 39 of the 40 failing gen-m9 cells whose arguments are literals or
Data cells, all 13 visibly-off ones among them, and H424/H429: the one-based rank `r = 1 + k*(n-1)`, `j = floor(r)`,
`h = r - j`, result `a[j] + h*(a[j+1] - a[j])` (sorted a[1..n]). The 1 added first is what loses a tiny k. The one miss is
gen-m9/5395b8a427764a3f Calc!A269 (Excel -2.499999999999999, the rule -2.4999999999999987); leave it in the baseline and
name it in the Outcome if no variant fits it without losing another cell. Details and cells: docs/excel-semantics/statistics.md
"PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1)". QUARTILE/QUARTILE.INC call the same
`percentile_inc_impl`, so they follow.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 42 known-failure lines in eval/baseline/gen-m9.txt, 13 visibly off.
Examples:
- gen-m9/17c110a53924beb8 Calc!A94 `=PERCENTILE.INC(Data!B7:D8,0.1)`: Excel -1.1102230246251565e-16, ours 0.0
- gen-m9/329b177b5d5165ad Calc!A323 `=PERCENTILE(Data!A1:B9,1E-10)`: Excel -9.999954990996277e-05, ours -9.999954991e-05
- gen-m9/a63d1c7d6f96c56d Calc!A318 `=NORM.INV(0.5,PERCENTILE(Data!B10:D12,0.999999),((-1)^DOLLARDE(Data!C4,18)))`: Excel 123456665.66621035, ours 123456665.66621037
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A7
- gen-m9/09b47629f54b9817.case.json (2): Calc!A120, A234
- gen-m9/17c110a53924beb8.case.json (1): Calc!A94
- gen-m9/1db18890916ee643.case.json (2): Calc!A325, A378
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A277
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A71
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A263, A323
- gen-m9/38e4fc309fe199ea.case.json (3): Calc!A1, A121, A311
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A99
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A78
- gen-m9/41248d1cb153c66e.case.json (2): Calc!A266, A390
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A269
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A188
- gen-m9/5be29fa316931f05.case.json (1): Calc!A175
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A65, A369
- gen-m9/607e85cfee83a689.case.json (1): Calc!A20
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A121
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A387
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A135, A259
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A205
- gen-m9/a4090567758520df.case.json (1): Calc!A28
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A227
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A318
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A16
- gen-m9/c4635963fc926284.case.json (1): Calc!A151
- gen-m9/cb61169f4c033d4b.case.json (2): Calc!A338, A370
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A234
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A120
- gen-m9/eccd03df42e40006.case.json (3): Calc!A48, A151, A162
- gen-m9/ede0323c389fc703.case.json (1): Calc!A350
- gen-m9/f82473275380882e.case.json (1): Calc!A163

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): the 13 visibly-off cells pass, and at least 39 of these 42 known-failure lines
      (the T-2256 cells in Context) disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), each one that stays
      named in the Outcome, with no new known-failure lines in any baseline.
- [ ] H424 and H429 leave eval/baseline/probes.txt, with no new F lines in any suite.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md with its probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERCENTILE.EXC and PERCENTRANK, unless the same probe shows them off.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): it now owns 13 visibly-off gen-m9 cells (Context), so the task is in the
campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria
still unmet then move to a follow-up task without the key.

Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: the 13 visibly-off gen-m9 cells of Context, e.g. gen-m9/38e4fc309fe199ea Calc!A311 `=PERCENTILE.INC(Data!B1:D6,1E-20)`: Excel 0, ours 1.1E-20; gen-m9/607e85cfee83a689 Calc!A20 `=PERCENTILE.INC(Data!B7:E11,1E-20)`: Excel 1E-15, ours 1.000011E-15.

## Outcome
`percentile_inc_impl` now interpolates from the one-based rank `1 + k*(n-1)` (docs/excel-semantics/statistics.md); QUARTILE/QUARTILE.INC follow. Eval delta: 41 of the 42 gen-m9 known-failure lines gone (all 13 visibly-off cells pass), probes H424/H429 gone, no new failures. gen-m9/5395b8a427764a3f Calc!A269 stays in the baseline (Excel -2.499999999999999, rule -2.4999999999999987), as the triage predicted.
