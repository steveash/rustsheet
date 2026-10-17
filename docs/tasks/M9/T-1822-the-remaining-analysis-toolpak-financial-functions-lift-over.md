---
id: T-1822
title: "The remaining Analysis ToolPak financial functions lift over arrays, after their later arguments are probed over ranges (SPEC-0011)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1372]
---

## Goal
Every remaining Analysis ToolPak financial function lifts over array literals in each scalar argument, and refuses a
multi-cell range there, as Excel does.

## Context
T-1372 entered EFFECT, NOMINAL, DOLLARDE, DOLLARFR, ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED,
YIELDDISC, YIELDMAT and the T-bills as `LiftArrays`. Left: ACCRINT, AMORDEGRC, AMORLINC, the COUP* family, CUMIPMT,
CUMPRINC, DURATION, MDURATION, FVSCHEDULE's principal, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PRICE, YIELD,
XIRR's guess and XNPV's rate. Their `ROWS(F(..{v;v}..))` cells in probes/fin-argument-modes are 2 in Excel and
`#N/IMPL!` here. SPEC-0011's probe-first guard applies: a later argument is entered only once it is probed over a
range on its own rows. probes/scalar-lift-open-a Y1-Y7 covers PRICE's frequency and basis, COUPDAYS's and COUPNUM's
frequency, COUPDAYS's basis and ACCRINT's frequency and basis; the other later arguments need a probe case like
probes/fin-later-argument-ranges (T-1372). Kernel work: PRICE/YIELD/ODD*YIELD iterate (fixed caps → `Weighted`), and
CUMIPMT/CUMPRINC loop from start to end period with no cap of their own (`CUMIPMT(1E-9,1E9,1,1,1E9,0)`), which needs
bounding first (as T-1820 for DB/VDB). XNPV and XIRR keep their values and dates `WholeByCaller` (T-1373, T-1821).

## Acceptance criteria
- [x] The later arguments are probed over a range (a probe case, excel-probe skill) and recorded in financial.md.
- [x] The lift cells of probes/fin-argument-modes for these functions pass (baseline lines removed); the 37 range
  `#VALUE!` cells still pass. Split by family if larger than M (ACCRINT, COUP*, DURATION, MDURATION, PRICE here; the
  rest are T-1841 to T-1843 and T-1845).
- [ ] `make check` green; reviewers approve. (`make check` green; the review is the PR step.)

## Out of scope
- Base-value ulp misses (T-1374, T-0901, T-0916).

## Outcome
- Probed first: probes/fin-remaining-later-argument-ranges (199 cells, excel-graph, 2026-10-17) puts every later
  argument of the remaining functions over a two-cell range on its own rows (H, all `#VALUE!`) beside its intersected
  control (I), plus CUMIPMT/CUMPRINC over 10^6-10^15 periods and AMOR* at period 1E9 (K1-K5, all computed by Excel;
  the 10^9-period CUMIPMT/CUMPRINC cells were probed but dropped from the case, the engine loops 10^9 times there)
  and three lifted element values. probes/fin-coupon-walk-bounds and -b (41 cells) probe the coupon schedule walk's
  bounds: a settlement below serial 1 is `#NUM!` in a 1900 workbook (financial.md "Coupon settlement before the
  calendar").
- ACCRINT, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, DURATION, MDURATION and PRICE are `LiftArrays`
  (functions/lift/kinds.rs, `// --- Financial ATP (T-1822) ---`), their kernels reading borrowed values in the old
  check order (MDURATION still reads yld and frequency first). Their schedule walks are bounded by the calendar once
  `Model::check_settlement` refuses a settlement below serial 1, and each element is charged `Weighted` per walk
  (`COUPON_WALK_WEIGHT` = 2^16, about 1 ms): 1,024 elements for COUP* and ACCRINT, 512 for DURATION, MDURATION and
  PRICE, 1-2.2 s at the calendar's longest schedule in release (financial.md "Coupon schedule work cap", DEVIATION).
- Scalar changes, evidence-driven: a settlement below 1900 serial 1 is `#NUM!` for the COUP* functions, PRICE, YIELD,
  DURATION and MDURATION (was a number, e.g. `COUPNUM(-1,43831,2)` 241; `COUPNUM(-1E15,43831,4)` walked about 10^6
  quarters). Error messages of the touched kernels are empty (the lift helpers); no value changes otherwise.
- Eval: probes 58 known failures fixed (fin-argument-modes 51, scalar-lift-open-a Y1-Y7), 0 regressions, 0 changed
  failing values cell by cell against main; the three new cases add 130 known failures (unlifted functions' H cells,
  T-1374's base-value ulps in the I controls, and the calendar-edge cells of T-1844).
- Follow-ups: T-1845 (CUMIPMT/CUMPRINC), T-1841 (AMOR*), T-1842 (YIELD, ODD*), T-1843 (XNPV's rate, XIRR's guess,
  FVSCHEDULE's principal), T-1844 (coupon schedules at the calendar's edges).
