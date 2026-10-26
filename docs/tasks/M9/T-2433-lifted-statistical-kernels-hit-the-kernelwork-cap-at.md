---
id: T-2433
title: "Lifted statistical kernels hit the KernelWork cap at a few hundred elements: #ERROR! where Excel spills (gen-m9, 9 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A lifted HYPGEOM.DIST, CHIDIST, CHISQ.INV.RT, GAMMADIST, POISSON or BETAINV over an array of a few hundred elements (a
PIVOTBY result, a TRANSPOSE of a range) returns its array, as Excel does (`#SPILL!` in these cells' layout), instead of
`#ERROR!` from the per-call lift budget.

## Context
Gap triage of the T-2255 gen-m9 sweep: 9 cells. `HYPGEOM.DIST(3,3,2,1,TRANSPOSE(Data!A1:D10))` (40 elements) is
`#ERROR!` in ours (also `ROWS(..)`), Excel `#SPILL!`; `CHIDIST(_xlfn.PIVOTBY(..),5)` and `CHISQ.INV.RT(_xlfn.PIVOTBY(..),10)`
(about 560 elements), `GAMMADIST(_xlfn.PIVOTBY(..),..)`, `KURT(POISSON(..,10000000001,_xlfn.PIVOTBY(..)),..)` (Excel
`#VALUE!`) and `BETAINV((Data!A1*EXPAND(Data!E8,2,Data!$D3)),..)` (Excel `#NUM!`). The budget is `KernelWork` per
function (lift/kinds/statistical.rs) against `limits::MAX_LIFTED_CELL_VISITS` (2^27): the weights of the
incomplete-gamma, quantile and hypergeometric kernels (2^19 terms or a search per element, T-1848, T-1872, T-1873) charge
the worst case, so a few hundred elements pass the cap although each element is cheap for these arguments. statistics.md
"Quantile searches", "Incomplete gamma" and "HYPGEOM.DIST: per-call cap and lifting" are DEVIATION entries (decision
0014): keep the bound (directive 5), but charge an element by its actual work (or a bound computed from its arguments),
so that these calls fit. Measure with benches/lift.rs before and after.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 9 known-failure lines in eval/baseline/gen-m9.txt, 9 formulas.
Examples:
- gen-m9/35e8168aae2b00f8 Calc!A22 `=BETAINV((Data!A1*EXPAND(Data!E8,2,Data!$D3)),Data!B6,(7+1.005))`: Excel #NUM!, ours #ERROR!
- gen-m9/3e98b26cf6713e77 Calc!A204 `=CHISQ.INV.RT(_xlfn.PIVOTBY(Data!A1:D12,Data!B1:B12,Data!C1:E12,LAMBDA(acc,(-0.5))),10)`: Excel #SPILL!, ours #ERROR!
- gen-m9/589f42e67f4b4b8d Calc!A121 `=KURT(POISSON((Data!D12*Data!B12),10000000001,_xlfn.PIVOTBY(Data!B1:B12,Data!C1:E12,Data!A1:D12,LAMBDA(x,x))),Data!E8:E12)`: Excel #VALUE!, ours #ERROR!
Cells:
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A22
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A62, A204
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A121
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A159
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A169
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A56
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A199, A359

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): Lifted calls inside financial functions that are `#SPILL!` in Excel and `#ERROR!` in ours (`ODDFYIELD(..,FILTER(..)-..,..)`, `AMORDEGRC(..,PIVOTBY(..),..)`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 visibly off.
Examples:
- gen-m9/ba4ddb817a8e700e Calc!A323 `=ODDFYIELD(45535,45661,(SQRTPI(3.14159)-AVERAGE(Data!E8:E10,Data!B5:B9,Data!C4)),38502,(FILTER(Data!B1:C12,(Data!B1:B12<>1e-15),Data!D10)-(Data!E6/3.14159)),82,110,4)`: Excel #SPILL!, ours #ERROR!
- gen-m9/e8d55ec393460bb7 Calc!A10 `=AMORDEGRC(56947,_xlfn.PIVOTBY(Data!B1:C12,Data!A1:C12,Data!B1:E12,LAMBDA(v,v),1,2),39763,716,2,PMT(100,271,417889,43,0),1.9)`: Excel #SPILL!, ours #ERROR!
Cells:
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A323
- gen-m9/e8d55ec393460bb7.case.json (1): Calc!A10

From T-2429 (2026-10-25): gen-m9/11eb3ab4f4f8104c Calc!A10 `=WEIBULL(HYPGEOM.DIST(1,2,10,7,_xlfn.GROUPBY(Data!B1:B12,Data!B1:E12,_xleta.MAX)),..)` is Excel `#SPILL!`, ours `#ERROR!`; the inner HYPGEOM.DIST lifted over the GROUPBY result is `#SPILL!` in Excel (probes/t2429/11eb3ab4f4f8104c A1), ours `#ERROR!`, likely the 32-element HYPGEOM.DIST lift charge (statistics.md "HYPGEOM.DIST: per-call cap and lifting"). Unverified; the cell stays in eval/baseline/gen-m9.txt.

## Acceptance criteria
Rescoped by T-2433's run (2026-10-26): the incomplete-gamma, quantile and financial charges have no closed-form bound
from their arguments (see T-2795), so this task covers HYPGEOM.DIST, and T-2795 owns the other cells (CHIDIST,
CHISQ.INV.RT, GAMMADIST, POISSON, ODDFYIELD, AMORDEGRC). gen-m9/35e8168aae2b00f8 A22 (BETAINV) had already left the
baseline on main.
- [x] Visible parity (gen-m9): the HYPGEOM.DIST known-failure lines (3e98b26cf6713e77 A62, 7bd219ec0c0c5e22 A159,
      a013a45a8fbcab4f A169, ee2497a5e683c5d9 A359, 11eb3ab4f4f8104c A10) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The per-call bound still holds for the worst cases the DEVIATION entries list (32 elements at the cap's bound;
      unit test `hypgeom_dist_is_charged_its_weight`), and the HYPGEOM.DIST DEVIATION entry says how an element is
      charged now.
- [x] `make check` green; reviewers approve.

## Out of scope
- The functions' values (their accuracy tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
HYPGEOM.DIST is charged by its counts (`ValueWork::HypergeometricSum`, lift/kinds/mod.rs; SPEC-0011 amended): the
larger of its walk (2^14 units) and its cdf sum's window, (min(sample_s, number_sample, population_s) + 132) * 16, at
most the cap's bound 2^22; a text count is left out of the min, an error or refused count is 1. 8,192 small elements
now fit where 32 did; 32 elements at the bound still do and 33 do not. benches/lift.rs (release): 32 at the bound
1.35-1.53 s (unchanged case), 8,192 4,000-step walks 0.99-1.0 s, 1,634 5,001-term sums 1.25-1.4 s. Eval: gen-m9 known
failures -5 (the five HYPGEOM.DIST cells), probes -1 (probes/t2429/11eb3ab4f4f8104c A1), no regressions. Follow-up:
T-2795 (incomplete gamma, quantiles, ODDFYIELD, AMORDEGRC).
