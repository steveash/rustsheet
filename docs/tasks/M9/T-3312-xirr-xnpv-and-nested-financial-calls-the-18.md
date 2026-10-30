---
id: T-3312
title: "XIRR, XNPV and nested financial calls: the 18 gen-m9 cells T-2611 left (numeric text in XIRR values, XNPV element order, inner-function causes)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The 18 gen-m9 known-failure lines T-2611 left give Excel's value: each one is traced to its cause, probed where the
order is unevidenced, and fixed here or moved to the task that owns the inner function.

## Context
Split from T-2611 (2026-10-30), which landed XIRR's guess-first order and the empty/numeric-text single-cell size checks
(financial.md "XIRR and XNPV under the gen-m9 statistical cells", T-2611 sentences). Remaining lines in
eval/baseline/gen-m9.txt (excel-graph, 2026-10-24), expected value in brackets:
- XIRR, top level:
  - 31474c20fc1dafd6 Calc!A351 `XIRR(Data!C9:C11,{39448,39508,39859},0.1)` (#NUM!, ours #VALUE!): C11 is the text "1".
    Text read as 1 (then all flows positive), skipped (then 2 values, 3 dates) or refused as `#NUM!` all fit; probe each
    reading separately (a numeric text in a range of mixed-sign flows with equal counts, and with one more date).
  - 30e20ba3302fea8b Calc!A208 `XIRR(-1.5,WEIBULL.DIST(0,1.5E+10,99999,FALSE))` (#NUM!, ours #N/A): ours computes the
    WEIBULL.DIST as 0; Excel's is probably `#NUM!` (99999^1.5E+10 overflows). Probe `WEIBULL.DIST(0,1.5E+10,99999,FALSE)`
    and hand it to functions-stat if so. Making two plain numbers `#NUM!` regressed 15 cells (b73c6cc96e093f03 A25,
    `XIRR(<number>,SLN(..))` is `#N/A`), so the XIRR order is not the cause.
- XNPV, top level: 1b60bf99d71424e7 Calc!A171 (#VALUE!: equal counts, text values C1:C2 and FALSE/empty dates C9:D9) and
  5ded7594e597e31e Calc!A12 `XNPV(1E+15,{-100;30;40;50},Data!D6:E7)` (#NUM!, a boolean in the dates): XNPV's element
  reading order of values and dates is unprobed.
- XIRR's text and boolean guess order (T-2611 review): ours reads a non-numeric text guess as `#VALUE!` before an error
  dates argument and the sizes, and a boolean guess as 1 or 0 that only skips the size checks; neither is evidenced.
  Probe `XIRR(<good values>,NA(),"x")`, `XIRR(<good values>,<other count>,TRUE)` and `XIRR(<good values>,NA(),TRUE)`
  (also an error values argument beside an error guess, and a numeric-text guess beside different counts).
- Nested (the inner call decides; trace each with a probe of the inner call):
  4bad303d32cc3f46 Calc!A364 (LOGNORM.INV over XIRR), 530c636749a85209 Calc!A338 (HYPGEOM.DIST over IRR),
  58e3253bbd3a7a82 Calc!A189 (EXPONDIST over XNPV), 5919660b7bb4d195 Calc!A144 (WEIBULL over XIRR),
  8f2ac25945af87da Calc!A137 (LOGNORMDIST over XNPV), a4090567758520df Calc!A277 (PEARSON over LOGEST over XNPV),
  b72d952c5f52cb4d Calc!A38 (ODDLYIELD over XIRR(MONTH(59085),Data!$B2)), cde551ea787f713a Calc!A37 (BETADIST over NPV).
- Values, not orders: 1b60bf99d71424e7 Calc!A181 (CUMPRINC, -90.53126715278742), 89f0aee953a55a45 Calc!A349
  (`FVSCHEDULE(5916,{-1E+300,1E+300})` is 0, ours `#DIV/0!`), 984cba67bbe6972b Calc!A141 (GAMMA.INV over FVSCHEDULE),
  a91bad4e84f49739 Calc!A158 (PPMT, -4.581304695604488e-303, an underflow).
Tool: a scratch suite holding one case's Data sheet with probe formulas in Calc runs through `rustsheet-eval --root <dir>
show p/x.case.json --all` (how T-2611 traced its cells).

## Acceptance criteria
- [ ] The 18 lines above disappear from eval/baseline/gen-m9.txt, or each one left is listed in the Outcome with its
      traced cause and owner task; no new known-failure lines.
- [ ] Every order changed is backed by a probe of the bare call and recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR/XNPV solver values.
