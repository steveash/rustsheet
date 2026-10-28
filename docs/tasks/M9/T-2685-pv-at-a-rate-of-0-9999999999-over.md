---
id: T-2685
title: "PV at a rate of -0.9999999999 over many periods: probe whether Excel gives #DIV/0! (two PROB cells left by T-2468)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PV at a rate just above -1 over many periods (`PV((-0.9999999999),161,-611,989,0)`, `PV((-0.9999999999),78,-250,878)`)
returns Excel's result, so the two PROB cells whose lower limit it is match Excel.

## Context
T-2468 matched PROB's data rules; two of its cells stay failing because their lower_limit is such a PV, which ours
gives as `#NUM!`. Excel's PROB result there is `#DIV/0!`, and every other PROB cell returns an error lower limit
as is, so Excel's PV is inferred to be `#DIV/0!` ((1+rate)^nper underflows to 0; statistics.md "PROB: an error data
argument first", UNVERIFIED). `ZTEST(Data!B10:B10,..,PV((-0.9999999999),167,-43,1E+300))` is `#DIV/0!` too
(a63d1c7d6f96c56d A324). Probe the PV calls themselves first (excel-probe skill), then fix PV in the financial
functions.
Cells:
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A394 `=PROB((ERF(1E-10)+(-0.5)),EFFECT(10,11),PV((-0.9999999999),161,-611,989,0))`: Excel #DIV/0!, ours #NUM!
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A70 `=PROB(Data!C6:C9,(NPV(123456789.123,Data!D7:D8)+BETAINV(0.5,0.5,100)),PV((-0.9999999999),78,-250,878))`: Excel #DIV/0!, ours #NUM!

## Acceptance criteria
- [ ] A probe records Excel's value of the two PV calls (and of a rate of -0.9999999999 over a few small nper).
- [ ] PV matches the probe; the 2 known-failure lines above disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] financial.md records the rule; statistics.md's PROB entry drops the UNVERIFIED PV note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PROB itself (T-2468).
