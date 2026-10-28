---
id: T-2906
title: "Probe the unowned calipers Mac-golden financial and statistical cells on excel-graph: formulas_financial, formula_stress_test, pv23_004 YIELD and pv26_014 IRR"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Every failing calipers cell listed below is classified with Excel for the web (the Phase 1 oracle of record,
decision 0039) as either an engine miss, which then has a functions-* owner, or a Mac-desktop-only difference,
which joins the desktop-only list (T-2663, M14). Today none of them has an open owner.

## Context
Gap triage 2026-10-27 (`rustsheet-eval fetch`, then `make eval` on origin/main 6087413b with calipers fetched).
The calipers goldens are Mac desktop Excel 16.03 caches (`excel-cached`), so a difference from them can be a
desktop-vs-web last-digit conflict (decision 0074, docs/excel-semantics/numbers.md "Excel for the web and desktop
Excel differ in the last digits") or a real miss. No open task names these workbooks or cells. T-2663 covers only
the CELL, INFO, trig and CHAR cells, and T-2708 covers formula_stress_test Statistical!B13.
Read each cell with `make eval-show CASE=calipers/<path>/golden.xlsx` (needs `target/release/rustsheet-eval fetch`).

Visibly off (more than half a unit in the 15th significant digit):
| case | cell | formula | Mac golden | ours |
|---|---|---|---|---|
| roundtrip/formulas_financial | Sheet1!D127 | `$C$52+NPV($C$10,$C$53,$C$54,$C$55,$C$56)` | -7175.324559303313 | -7175.324559303306 |
| roundtrip/formulas_financial | Sheet1!C133 | `IRR($C$52:$C$56,0.1)` | 0.07713847295204346 | 0.07713847295204368 |
| roundtrip/formulas_financial | Sheet1!D186 | `XNPV($C$7,$C$52:$C$56,$C$42:$C$46)` | 6476.818236047038 | 6476.8182360470455 |
| pv_goldens/pv23_004 | Bonds-Calls-Puts!Q20, Q23 | `YIELD(Settlement_Date,Q32,Coupon_Rate,...)` | 0.05748925606854341 | 0.05748925606854292 |
| pv_goldens/pv26_014 (and pv26_2_014) | Returns!D63 | `IRR(E63:J63)` | 5.753593644477435 | 5.75359364447743 |
| roundtrip/formula_stress_test | Finance!B45 | `ODDFYIELD(D45,...,L45)` | 0.07724554159729888 | 0.07724554159781744 |
| roundtrip/formula_stress_test | Statistical!B17 | `_xlfn.CONFIDENCE.T(C17,D17,E17)` | 0.10418907592517826 | 0.10418907592517064 |
| roundtrip/formula_stress_test | Statistical!B42, B46 | `_xlfn.NORM.INV(B41,D41,E41)`, `NORMINV(B45,D41,E41)` | 8.881784197001252e-16 | 6.661338147750939e-16 |
| roundtrip/formula_stress_test | Statistical!E138 | spilled member of `LINEST(D129:I129,D128:I128,TRUE,TRUE)` (D138) | 0.1999999999999993 | 0.20000000000000462 |

Last-digit (within 15 digits) in the same workbooks, also unowned:
- formulas_financial: PMT C67, E67, G67, C68, D68, D69, C70, E70, C71, D71, C76, E76, G76, H76, C77, D77, E77, F77,
  G77, C79, E79, C80 (e.g. `PMT($C$6,$C$11,$C$18,0,0)` golden 10099.999999999998, ours 10100); NPV C124; IRR C132,
  D132, C134; XNPV C186, E186.
- pv23_004 Bonds-Calls-Puts: YIELD L20, N20, P20, R20, V20, X20, L23, N23, P23, R23, V23, X23, Q27.
- pv26_014 and pv26_2_014 Returns: IRR D44, D53, D54, D65, D66 (and the Sensitivity cells that copy D54).
- formula_stress_test: Finance B10, B17, B20, B21, B25, B32; Statistical B6, B8, B10, B11, B14-B16, B23-B26,
  B29-B32, B33, B37, B39, B40, B49, B50, B54, B55, B57, B77, B118-B123, B130, B138-B144 (LINEST, LOGEST, TREND
  members). Some functions already have Phase 1b owners (PMT and NPV T-1528, LINEST T-2006): once a cell is shown
  to be an engine miss, name it in that owner's task or a new one rather than duplicating it.

Method: one probe case, probes/t2906-calipers-financial-stat (excel-probe skill, excel-graph), that rebuilds each
listed formula with the golden's own inputs as literals or input cells (copy the values from `rustsheet-eval
snapshot`, at 17 significant digits; keep defined names such as Settlement_Date as cells). Do not upload the
calipers workbooks themselves: the suite is scored from their cached values, and some inputs were anonymised
(decision 0133).

## Acceptance criteria
- [ ] probes/t2906-calipers-financial-stat is committed with excel-graph values for every cell listed above
      (one probe cell per distinct formula and input set; copies such as the Sensitivity cells need none).
- [ ] The Outcome has a table: each listed calipers cell, the web value, and whether web equals ours, the Mac
      golden, or neither.
- [ ] Cells where web equals the Mac golden and not ours are engine misses: each is named, with case and cell,
      in an open functions-* task (an existing owner of the function where one fits, else a new task filed with
      `make new-task`; `campaign: visible` and a `## Visible-parity stage` for the visibly-off ones,
      `defer: phase-1b` for last-digit ones).
- [ ] Cells where web equals ours are listed in docs/excel-semantics/numbers.md "Excel for the web and desktop Excel
      differ in the last digits" as Mac-desktop conflicts, citing the probe cells and the calipers cells, and added
      to T-2663's list (if T-2663 is not claimed by a pushed branch; else file a new M14 research task).
- [ ] The new probe's own known failures are recorded with `make eval-baseline`, each with an owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function (the owner tasks do that).
- The calipers cells T-2663 (CELL, INFO, trig, CHAR), T-2704 (formulas_nested), T-0454 (pv26_014 Sensitivity row 39
  data table), T-0458/T-0463 (pv23_007 and other PPMT-fed cells) and T-2708 (formula_stress_test B13) own.
