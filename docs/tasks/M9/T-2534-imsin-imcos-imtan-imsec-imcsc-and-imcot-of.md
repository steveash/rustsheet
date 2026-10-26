---
id: T-2534
title: "IMSIN, IMCOS, IMTAN, IMSEC, IMCSC and IMCOT of a real number take SIN, COS and TAN's kernels and limit (gen-m9, 62 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1966]
campaign: visible
---

## Goal
IMSIN, IMCOS, IMTAN, IMSEC, IMCSC and IMCOT of a real number (imaginary part 0) give the worksheet SIN, COS and TAN values (and their reciprocals) and are `#NUM!` where those are, except that IMCOT of 1E+200 is "0".

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). RustSheet's own SIN, COS and TAN (`excel_sin_cos`, `excel_tan`, the x87 reduction of numbers.md "Trig of a huge angle") reproduce every Excel value here, libm does not: `IMTAN("5")` -3.38051500624659 (35 cells), `IMSIN(123456789.123)` 0.99984293955201, `IMCOS(..)` 0.0177227601686489, `IMTAN(..)` 56.4157574800738, `IMSEC(..)` 56.424619556099, `IMCOT(..)` 0.0177255441505541, `IMSIN("710")` 0.0000602887066915739, `IMCOT("710")` 16586.8543722202, `IMCSC("710")` 16586.8544023646, `IMSIN(3.14159)` 2.65358979335278E-06, `IMTAN(3.14159)` -2.65358979336212E-06. Past the limit (2^27) the result is `#NUM!`: "1E+200" in IMSIN, IMTAN, IMSEC and IMCSC, `IMSIN("7777777777")`, IMSEC of 1E+15, `IMTAN("1.05966105763914E+57")`; but `IMCOT("1E+200")` is "0" (gen-m9/060b18586fcff6e6 A119; ours -1.18802637371944).
Builds on T-1966 (done on origin/agent/functions-engineering/T-1966-probe-imtan-imcot, not merged): it moved IMCOT of a real to `real_tan` and IMTAN/IMCOT's double angle to `sin_cos_double_angle`, and its draft follow-up (numbered T-2185 on that branch, an id main has since used for a VDB task) asked to decide IMTAN of a real because COMPLEXs.xlsx N6 (Mac cache) has `IMTAN(5)` -3.38051500624658. These 35 excel-graph cells decide it for the web (decision 0039: excel-graph is the oracle of record); if ironcalc-calc's N6 then fails, record it as a web-vs-desktop conflict as decision 0039 and docs/excel-semantics/README.md prescribe (never hand-edit eval/baseline), and drop the branch's draft. Complex arguments are out of scope except where they share the code path (IMEXP already uses `excel_sin_cos`).
Code: `imsin`, `imcos`, `imsec`, `imcsc`, `tan_parts`/`imtan`, `imcot` in crates/rustsheet-engine/src/functions/engineering/complex.rs; `excel_sin_cos`, `excel_tan`, `EXCEL_TRIG_LIMIT` in functions/math_and_trigonometry.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 62 known-failure lines in eval/baseline/gen-m9.txt, 22 formulas, 62 visibly off.
Examples:
- gen-m9/01810fc89c5ec2c0 Calc!A18 `=IMTAN("5")`: Excel "-3.38051500624659", ours "-3.38051500624658"
- gen-m9/060b18586fcff6e6 Calc!A119 `=IMCOT("1E+200")`: Excel "0", ours "-1.18802637371944"
- gen-m9/1068c44afcd56d53 Calc!A57 `=IMSIN(HEX2OCT("FFFFFFFFFF"))`: Excel #NUM!, ours "0.126310536650214"
- gen-m9/1068c44afcd56d53 Calc!A262 `=IMSEC(Data!B6)`: Excel "56.424619556099", ours "56.4246195623743"
- gen-m9/12e4aa13a20cc1f8 Calc!A216 `=IMTAN("1E+200")`: Excel #NUM!, ours "-0.84173215521237"
- gen-m9/3cf48801617a72ad Calc!A145 `=IMCOT("710")`: Excel "16586.8543722202", ours "16586.8543722171"
Cells:
- gen-m9/01810fc89c5ec2c0.case.json (2): Calc!A18, A116
- gen-m9/060b18586fcff6e6.case.json (2): Calc!A16, A119
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A270
- gen-m9/1068c44afcd56d53.case.json (3): Calc!A57, A219, A262
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A216
- gen-m9/3cf48801617a72ad.case.json (2): Calc!A81, A145
- gen-m9/408aa11aa923a4d9.case.json (2): Calc!A67, A114
- gen-m9/40fc68fccfaf9b76.case.json (2): Calc!A258, A350
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A360
- gen-m9/516488da78a6b08c.case.json (3): Calc!A88, A160, A182
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A220
- gen-m9/5bdef392d16a5efb.case.json (4): Calc!A20, A153, A277, A324
- gen-m9/65dc4e4da669f98a.case.json (2): Calc!A126, A314
- gen-m9/65ff9a692665eba9.case.json (2): Calc!A100, A345
- gen-m9/6d16afd40600302b.case.json (1): Calc!A53
- gen-m9/74779cff2810ccf4.case.json (1): Calc!A154
- gen-m9/76b07b57099ccc8d.case.json (2): Calc!A48, A312
- gen-m9/7820ebc42c7a56a6.case.json (2): Calc!A72, A176
- gen-m9/79dac8dde81afda9.case.json (3): Calc!A20, A154, A201
- gen-m9/7a819ffbbeff8fcc.case.json (4): Calc!A215, A297, A313, A362
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A35
- gen-m9/831f987e63e2412c.case.json (1): Calc!A276
- gen-m9/895a33aa8352864a.case.json (3): Calc!A32, A69, A163
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A271
- gen-m9/9fa170a9f17624fd.case.json (1): Calc!A97
- gen-m9/a1b4181c04804efc.case.json (2): Calc!A9, A254
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A268
- gen-m9/c2d778463fe008ed.case.json (1): Calc!A256
- gen-m9/ca182d96dedee546.case.json (1): Calc!A205
- gen-m9/cb80ca09be0e12bc.case.json (1): Calc!A118
- gen-m9/e707d43a891ba9b5.case.json (1): Calc!A17
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A109
- gen-m9/e882060cf4613302.case.json (1): Calc!A146
- gen-m9/ef33be4fd2389f3c.case.json (2): Calc!A225, A267
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A225
- gen-m9/fb73d8163ad87fc6.case.json (2): Calc!A45, A363

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 62 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline. Partly: 21 leave; the 41 IMTAN-of-a-real
      cells move to T-2774 (blocked on the N6 web-vs-desktop conflict, T-1075 item 17); see Outcome.
- [x] Unit tests (IMTAN of 5 keeps the desktop value, T-2774): IMSIN/IMCOS/IMTAN of 5, 710 and 123456789.123 equal SIN/COS/TAN's text; "1E+200" is `#NUM!` in all but IMCOT, which is "0".
- [x] engineering.md's entry gets an "Ours" line; COMPLEXs.xlsx N6 is handled as above if it moves.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSINH and the exp-based sinh/cosh (T-2535); IMSQRT (T-2536).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
- IMSIN, IMCOS, IMSEC, IMCSC and IMCOT of a real number take SIN, COS and TAN's kernels and are `#NUM!` from |x| = 2^27,
  IMCOT "0" there (`real_trig` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs). IMTAN takes the limit.
- New probe probes/t2534-im-trig-real-limit (60 cells, excel-graph): the limit is 2^27 for all six (134217727 answers).
- IMTAN of a real below the limit keeps the double-angle form: switching it to TAN's kernel regresses ironcalc-calc
  COMPLEXs.xlsx N6 (Mac cache `IMTAN(5)` -3.38051500624658), and recording that needs a decision record plus
  `--allow-regressions`, which this run could not take. Split to T-2774 (blocked, T-1075 item 17 added). engineering.md
  records the conflict as CONFLICTING.
- Eval delta: gen-m9 known failures 4168 → 4147 (fixed 21); probes 2781 → 2783 (fixed 2 in eng-imtan-imcot-overflow,
  new coverage 4: the new probe's IMTAN-of-a-real cells D1, D8-D10, owned by T-2774). Regressions 0.

