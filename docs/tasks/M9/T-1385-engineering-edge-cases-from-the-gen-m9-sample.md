---
id: T-1385
title: "Engineering edge cases from the gen-m9 sample: CONVERT/DELTA/GESTEP error order, HEX2OCT negative places, IMSQRT(), Bessel fractional negative order, IM* E-notation parts"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The engineering-function differences that the seeded engineering sample from T-1085 turned up are
probed one question per cell, recorded in docs/excel-semantics/engineering.md, and fixed where the
evidence is clear. Any part that needs more than size M is split into its own task.

## Context
The sample was `rustsheet-gen --milestone M9 --count 20 --seed 17`, using a scratch catalog that held
only the engineering signatures. It was not committed. excel-graph accepted all 20 workbooks, and the
engine passed 744 of 800 cells. The misses below are not covered by T-1383 (lifting), T-1384
(IMSUM/IMPRODUCT, booleans in IM*), T-0907 (IM* last digit) or T-0900/T-0908 (Bessel precision).
Each one is a hypothesis drawn from a generated formula, so probe it before you implement it:
- **CONVERT reads its number before its units.** A text number with an unknown unit is `#VALUE!`,
  where we give `#N/A`: `ERF.PRECISE(CONVERT(<text>,"ft",<number>)+..)`.
- **DELTA and GESTEP may report the second argument's error first.**
  `DELTA(<#NUM!>,CONVERT(0.2,"K","m"))` is `#N/A`, where we give `#NUM!`. `GESTEP(<#VALUE!>,<#N/A>)` is
  `#N/A`, where we give `#VALUE!`.
- **A boolean cell in BITAND, DELTA or GESTEP is `#VALUE!`.** We give `#NUM!`:
  `DELTA(GESTEP(BITAND(<bool cell>,139)),..)`.
- **HEX2OCT of a negative number ignores `places`.** `HEX2OCT("FFFFFFFFFF",4)` is "7777777777", where we
  give `#NUM!`.
- **IMSQRT of an empty string is `#NUM!`.** We give "0" for `IMSQRT(""&"")`.
- **A fractional negative Bessel order is `#NUM!`.** `BESSELK(5,-0.0001)` and `BESSELK(x,BESSELY(14,5))`
  are `#NUM!`, but we truncate the order to 0 and return a value.
- **IMPOWER's tiny parts are written in E notation, and their value comes from a different
  computation.** `IMPOWER("2i",-1)` is "3.06287113727155E-17-0.5i", where we give
  "0.0000000000000000306161699786838-0.5i". `IMPOWER("-i",4)` is "1+2.45029690981724E-16i".

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3462856259a37891 Calc!A25 `=(..&IMPOWER("3+4i",1e-15))`: Excel "1+9.27295218001614E-16i", engine "1+0.000000000000000927295218001614i" (the IMPOWER E-notation item).

## Acceptance criteria
- [ ] A probe case answers each item above, and its findings are in docs/excel-semantics/engineering.md.
- [ ] The items with clear evidence are fixed, with no eval regressions. Any item left open is filed as
      its own task.
- [ ] gen-m9/3462856259a37891 A25 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bessel and IM* last-digit precision (T-0900, T-0908, T-0907).
