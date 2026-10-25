---
id: T-2335
title: "IMCSC, IMSEC, IMSECH keep tiny results (1E-228..1E-246) instead of flushing to 0"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IMCSC, IMSEC and IMSECH keep tiny results where ours flushes to 0, matching Excel.

## Context
Found by T-2043's sweep: `IMCSC("-724.227-562.884i")` is -6.94796703439417E-245 / -6.26263832046024E-246 (sweep E55, F55), IMSEC of it H55, I55, and `IMSECH("-524.561+5.3113i")` is 1.73046857535442E-228 / -2.53533899849465E-228 (K42, L42); ours gives 0. Needs a rule for when the denominator overflow yields 0 vs a tiny value (the existing note says IMSEC/IMCSC/IMSECH answer 0 where the denominator overflows). Numeric-kernel question, hence opus.

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258; engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"): `IMCSC("-724.227-562.884i")` (3 cells) and `IMSEC(..)` (5) again give the 1E-245 parts (ours "0"), and `IMEXP("-724.227-562.884i")` is "0" (ours "0+0i"): e^-724.227 is subnormal and both parts flush, unlike IMSEC's normal tiny parts (1 cell). The rule this task writes must keep a normal tiny part and flush a subnormal one.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 9 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas, 9 visibly off.
Examples:
- gen-m9/1068c44afcd56d53 Calc!A1 `=IMCSC("-724.227-562.884i")`: Excel "-6.94796703439417E-245-6.26263832046024E-246i", ours "0"
- gen-m9/493bb194c8e0b577 Calc!A47 `=IMSEC("-724.227-562.884i")`: Excel "-6.26263832046024E-246+6.94796703439417E-245i", ours "0"
- gen-m9/b5aef3645cec1b95 Calc!A154 `=IMEXP("-724.227-562.884i")`: Excel "0", ours "0+0i"
Cells:
- gen-m9/1068c44afcd56d53.case.json (2): Calc!A1, A393
- gen-m9/493bb194c8e0b577.case.json (1): Calc!A47
- gen-m9/7096c4cd615d43ac.case.json (1): Calc!A25
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A82
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A154
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A132
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A135
- gen-m9/fb73d8163ad87fc6.case.json (1): Calc!A228

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 9 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] E55, F55, H55, I55, K42, L42 of the sweep pass without regressing the overflow-to-0 cells already matched.
- [x] `make check` green; baseline updated; reviewers approve.

## Out of scope
- Last-digit kernels (T-2044).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
Where the denominator of IMSEC, IMCSC or IMSECH overflows, the result is now the trigonometric parts
times `2e^-|v|` (`overflowed_reciprocal` in complex/trig.rs), which reproduces Excel's 15 digits for
the six sweep cells; a subnormal result part prints as 0 in the complex number-to-text (IMEXP "0").
Eval: gen-m9 known failures 5294 → 5285 (9 fixed, 0 regressions); probes-numeric
im-last-digit-sweep 1385 → 1394 of 1404 cells (E55:I55, D55, J42:L42). IMCSCH's large non-real
arguments stay 0 (unprobed; noted in engineering.md).
