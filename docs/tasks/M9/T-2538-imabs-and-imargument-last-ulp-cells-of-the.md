---
id: T-2538
title: "IMABS and IMARGUMENT last-ulp cells of the gen-m9 engineering sweep, after T-2044 (27 cells)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2044]
defer: phase-1b
---

## Goal
After T-2044, IMABS and IMARGUMENT match the gen-m9 sweep's cells to the bit.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. None of these is visibly off (each is within half a unit of the 15th digit), so this task has no campaign key. `IMABS("0.25-3i")` is 3.0103986446980735 (24 cells) where both `sqrt(x²+y²)` and `hypot` give 3.010398644698074: on excel-graph too IMABS is neither (engineering.md had blamed the Mac cache); `IMABS("-724.227-562.884i")` 917.24759197558 (ours 917.2475919755799); `IMABS(IMCOS("0.25-3i"))` 10.064621669102777. `IMARGUMENT("1.5E+2-2E-3i")` is -1.3333333332543209E-05, which the engine's worksheet ATAN2 gives (`ATAN2(150,-0.002)`) and f64::atan2 does not (-1.333333333254321E-05), and `IMARGUMENT(IMSECH("-1.5+0.5i"))` 0.4592256782430556. T-2044 (in progress on its branch) owns the fit of both kernels; docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)".

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 27 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas, 0 visibly off.
Examples:
- gen-m9/09e6a8028f5acfae Calc!A14 `=IMABS("0.25-3i")`: Excel 3.0103986446980735, ours 3.010398644698074
- gen-m9/4fffa5addab1f8af Calc!A91 `=IMABS(IMCOS("0.25-3i"))`: Excel 10.064621669102777, ours 10.06462166910278
- gen-m9/a7fda34502ed3030 Calc!A147 `=IMABS("-724.227-562.884i")`: Excel 917.24759197558, ours 917.2475919755799
- gen-m9/df083d85b436b49a Calc!A55 `=IMARGUMENT(IMSECH("-1.5+0.5i"))`: Excel 0.4592256782430556, ours 0.45922567824305555
- gen-m9/f8bc18a0e45a0e1a Calc!A183 `=IMARGUMENT("1.5E+2-2E-3i")`: Excel -1.3333333332543209e-05, ours -1.333333333254321e-05
Cells:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A14
- gen-m9/2b0324e2de2e7b74.case.json (2): Calc!A108, A202
- gen-m9/3cf48801617a72ad.case.json (1): Calc!A324
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A317
- gen-m9/493bb194c8e0b577.case.json (2): Calc!A39, A133
- gen-m9/4e180f54ff140abc.case.json (1): Calc!A320
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A91
- gen-m9/516488da78a6b08c.case.json (1): Calc!A332
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A269
- gen-m9/7096c4cd615d43ac.case.json (2): Calc!A152, A246
- gen-m9/76b07b57099ccc8d.case.json (1): Calc!A59
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A220
- gen-m9/831f987e63e2412c.case.json (1): Calc!A301
- gen-m9/895a33aa8352864a.case.json (1): Calc!A139
- gen-m9/a7fda34502ed3030.case.json (2): Calc!A147, A382
- gen-m9/ca182d96dedee546.case.json (1): Calc!A105
- gen-m9/df083d85b436b49a.case.json (2): Calc!A55, A221
- gen-m9/ef33be4fd2389f3c.case.json (1): Calc!A368
- gen-m9/f280df8ba95f96b9.case.json (2): Calc!A213, A260
- gen-m9/f8bc18a0e45a0e1a.case.json (2): Calc!A28, A183

## Acceptance criteria
- [ ] These 27 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] The im-last-digit-sweep Calc!B33 cell added by gap triage 2026-10-30 is bit-exact (`make eval-show`; probes-numeric is ungated), or the Outcome names it with the hypot forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: one IMABS cell of the probes-numeric IM sweep, 1 ulp off, that no open task names. Same modulus computation as this task's IMABS cells. Calc!B33 `=IMABS(A33)` with A33 `"956.615+0.00387493i"`: Excel 956.6150000078482, ours 956.6150000078479.
- probes-numeric/im-last-digit-sweep.case.json (1): Calc!B33
