---
id: T-1822
title: "The remaining Analysis ToolPak financial functions lift over arrays, after their later arguments are probed over ranges (SPEC-0011)"
milestone: M9
lane: functions-financial
status: in-progress
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
- [ ] The later arguments are probed over a range (a probe case, excel-probe skill) and recorded in financial.md.
- [ ] The lift cells of probes/fin-argument-modes for these functions pass (baseline lines removed); the 37 range
  `#VALUE!` cells still pass. Split by family if larger than M.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Base-value ulp misses (T-1374, T-0901, T-0916).
