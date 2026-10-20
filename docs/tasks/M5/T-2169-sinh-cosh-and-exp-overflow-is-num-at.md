---
id: T-2169
title: "SINH, COSH and EXP overflow is #NUM! at once (no infinity reaches a caller); SECH past the overflow is 0"
milestone: M5
lane: functions-math
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
SINH and COSH are `#NUM!` wherever `e^|x|` overflows (|x| = 710 is past it), SECH and CSCH there are 0, and no SINH, COSH or EXP result that overflowed reaches an enclosing function as a number: the error is the function's own result, as Excel does.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Two engine bugs, one fix site (`crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs`, `hyperbolic.rs`):
1. `hyperbolic::sinh`/`cosh` switch to libm `x.sinh()`/`x.cosh()` above `EXP_SAFE_LIMIT`, which stay finite up to 710.47, so
   SINH(710) is 1.117E+308 where Excel says `#NUM!`, and SECH(710) = 1/COSH is 8.95E-309 where Excel says 0.
2. SINH, COSH and EXP return an infinity inside `number_fn`/`number_result` (lift/mod.rs), which only maps NaN to `#NUM!`; the cell
   store turns a top-level infinity into `#NUM!`, but a caller sees a number: `ISNUMBER(SINH(1E+9))` is TRUE, `TEXTJOIN` prints
   "inf", `ATAN(COSH(1E+15))` is pi/2, `MOD(2^53+1,SINH(1E+9))` is 2^53, `ISODD(EXP(2^53+1))` TRUE.
Rule and evidence: docs/excel-semantics/numbers.md "SINH and COSH overflow is `#NUM!` at once; SECH there is 0" (added by this
triage) and "Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds" (SINH(709.78) is finite, CSCH(710) = 0). T-1944 probes
EXP's own overflow edge (the last finite argument); this task only needs an infinite result to be `#NUM!` at once. Check other
`number_fn` users for the same leak (a general non-finite check in `number_fn` is the natural place).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 37 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/043ee562f32eb901 Calc!A183 | `=REPLACEB(IF(ISERR(1/0),PRODUCT(Data!A11:A12,Data!E9:E12,Data!D$1),(Data!$C9^Data!A$6)),(SINH(710)...` | "#NUM!" | "#VALUE!" |
| gen-m5/109fc90dceb50ba9 Calc!A257 | `=SINH((-710))` | "#NUM!" | -1.1169973830808557e+308 |
| gen-m5/1962de140dae0111 Calc!A30 | `=SECH(710)` | 0.0 | 8.95257245135026e-309 |
| gen-m5/1b30e4d075b252ea Calc!A318 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/25768de48ae7f2f9 Calc!A315 | `=SINH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/2abdc1524e5aa685 Calc!A133 | `=COSH((-710))` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/2abdc1524e5aa685 Calc!A313 | `=GCD(SIN(((-1)*Data!C7)),SINH(134217728),(NUMBERVALUE(Data!$A6,Data!A6,Data!E1)/(Data!A11*Data!$B9)))` | "#NUM!" | "#VALUE!" |
| gen-m5/38b1b8e51dbad326 Calc!A331 | `=ATAN(COSH(1E+15))` | "#NUM!" | 1.5707963267948966 |
| gen-m5/399071ff6782ab16 Calc!A3 | `=ISERR(COSH((-710)))` | true | false |
| gen-m5/4febabc66cfdf9a1 Calc!A156 | `=ARRAYTOTEXT(SINH(134217727),2)` | "#NUM!" | "#VALUE!" |
| gen-m5/50d8a085335c05e6 Calc!A296 | `=COSH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/5704666097cbc5b2 Calc!A249 | `=SINH((-710))` | "#NUM!" | -1.1169973830808557e+308 |
| gen-m5/6361fdf5993da954 Calc!A153 | `=SECH(710)` | 0.0 | 8.95257245135026e-309 |
| gen-m5/6361fdf5993da954 Calc!A284 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/651e713f5ae232ed Calc!A260 | `=SECH(710)` | 0.0 | 8.95257245135026e-309 |
| gen-m5/6a511beadb56b932 Calc!A107 | `=GCD(EXP((Data!B10+Data!B2)),Data!A8:D9,SQRT(0.1))` | "#NUM!" | "#VALUE!" |
| gen-m5/6a8bb7cd6d7769ad Calc!A41 | `=SINH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/6ba5e4aceeef8209 Calc!A316 | `=TEXTJOIN(IFS(Data!A$12,MAX(Data!C4:C11,0.3)),(Data!E6>=42),COSH(1E+9))` | "#NUM!" | "inf" |
| gen-m5/901e91c09b347914 Calc!A87 | `=COSH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/90bf45bea539723d Calc!A86 | `=ISODD(EXP(2^53+1))` | "#NUM!" | true |
| gen-m5/9284730052aff46c Calc!A396 | `=XOR(FALSE,ISNUMBER(SINH(134217728)))` | false | true |
| gen-m5/97737fad5004ba83 Calc!A21 | `=MOD(ACOT(SINH(1E+15)),INDEX(Data!B2:C5,2,Data!$D7))` | "#NUM!" | "#REF!" |
| gen-m5/ae9ed30812c4d62c Calc!A182 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/b0ecb7ca78460b77 Calc!A246 | `=SINH((-710))` | "#NUM!" | -1.1169973830808557e+308 |
| gen-m5/bbbd316c346ce645 Calc!A315 | `=SECH(710)` | 0.0 | 8.95257245135026e-309 |
| gen-m5/bbbd316c346ce645 Calc!A327 | `=MOD(2^53+1,SINH(1E+9))` | "#NUM!" | 9007199254740992.0 |
| gen-m5/c0651f0e0fb544d0 Calc!A291 | `=SINH((-710))` | "#NUM!" | -1.1169973830808557e+308 |
| gen-m5/d0419fa09d6a756a Calc!A338 | `=SINH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/d1b251fd6e9c4b4b Calc!A276 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/d9fb1e918971111b Calc!A62 | `=TEXTJOIN("ü",EXACT(TEXT("?b","h:mm:ss AM/PM"),CHAR(10)),COSH(MIN(Data!D11:D11)))` | "#NUM!" | "inf" |
| gen-m5/d9fb1e918971111b Calc!A108 | `=VALUETOTEXT(((Data!E11-123456789.123)-SINH((-710))),0)` | "#NUM!" | "1.1169973830809E+308" |
| gen-m5/d9fb1e918971111b Calc!A226 | `=SINH(710)` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/d9fb1e918971111b Calc!A275 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/eb942f4a59b1c004 Calc!A393 | `=SECH((-710))` | 0.0 | 8.95257245135026e-309 |
| gen-m5/ecd1b828aebad1e8 Calc!A185 | `=CEILING((Data!B11-SUM(Data!E1:E9,Data!A4:A4)),((Data!A11+Data!B2)^SECH(710)))` | -1000000000000000.0 | "#NUM!" |
| gen-m5/f0f205865a3a4328 Calc!A263 | `=COSH((-710))` | "#NUM!" | 1.1169973830808557e+308 |
| gen-m5/fc81c2f704bef138 Calc!A46 | `=SUMXMY2(Data!A4:A4,EXP(2958465))` | "#NUM!" | "#DIV/0!" |

## Acceptance criteria
- [ ] These 37 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] numbers.md entry marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The exact SINH/COSH cut inside (709.78, 710) and EXP's overflow edge (T-1944).
- The SINH/TANH kernel on (0, 1) (T-2134).

## Outcome
`hyperbolic::{sinh,cosh}` now overflow to infinity at ±710 (SECH/CSCH give 0), and `number_fn` maps any non-finite result to `#NUM!`, so SINH/COSH/EXP overflow no longer reaches an enclosing function as a number. numbers.md entry marked implemented. Eval: 36 known-failure lines removed from eval/baseline/gen-m5.txt (the task listed 37); no new failures.
One listed cell still fails, gen-m5/4febabc66cfdf9a1 Calc!A156 (`ARRAYTOTEXT(SINH(134217727),2)`, Excel `#NUM!`): SINH itself is now `#NUM!`, so the remaining difference is in how ARRAYTOTEXT takes an error argument, not in this task's overflow rule. TODO(T-2169) follow-up: file a task for ARRAYTOTEXT with an error scalar argument.
