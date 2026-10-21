---
id: T-2170
title: "ASIN and ACOS are #NUM! just below -1 (ASIN(-1-2^-52))"
milestone: M5
lane: functions-math
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
ASIN and ACOS of an argument just below -1 (e.g. -1-2^-52) are `#NUM!`, as Excel does.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `excel_asin` (mathematical.rs) computes `y = 1 - x`; for x = -1.0000000000000002 that rounds to exactly 2, the radicand
`y*(2-y)` is 0 and `atan(x/0)` gives -pi/2 (ACOS pi). Excel is `#NUM!` (out of domain), also nested. Rule: docs/excel-semantics/numbers.md
"Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once" (paragraph "Just below -1 as well", added by this triage). An explicit
`|x| > 1` check before the kernel is enough; ASIN(1+2^-52) is already `#NUM!`.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 14 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/38b1b8e51dbad326 Calc!A222 | `=ACOS((-1-2^-52))` | "#NUM!" | 3.141592653589793 |
| gen-m5/3b813881f835cd90 Calc!A273 | `=ASIN((-1-2^-52))` | "#NUM!" | -1.5707963267948966 |
| gen-m5/49bfbf67a1409563 Calc!A378 | `=ACOS((-1-2^-52))` | "#NUM!" | 3.141592653589793 |
| gen-m5/50d8a085335c05e6 Calc!A226 | `=ASIN(MAX(ASIN((-1-2^-52)),Data!A4:A10,Data!C4:C4))` | "#NUM!" | 1.5707963267948966 |
| gen-m5/90bf45bea539723d Calc!A236 | `=LOG(ACOS((-1-2^-52)))` | "#NUM!" | 0.49714987269413385 |
| gen-m5/97737fad5004ba83 Calc!A298 | `=ACOS((-1-2^-52))` | "#NUM!" | 3.141592653589793 |
| gen-m5/a6fec430ac350c1a Calc!A140 | `=REPLACE((Data!C$12&Data!D9),ROUNDDOWN(Data!E3,9),(ACOS((-1-2^-52))-ROUNDDOWN(2^32,10)),IFERROR(UN...` | "#NUM!" | "#VALUE!" |
| gen-m5/ae9ed30812c4d62c Calc!A217 | `=ASIN((-1-2^-52))` | "#NUM!" | -1.5707963267948966 |
| gen-m5/ae9ed30812c4d62c Calc!A301 | `=ACOS((-1-2^-52))` | "#NUM!" | 3.141592653589793 |
| gen-m5/ae9ed30812c4d62c Calc!A348 | `=ASIN((-1-2^-52))` | "#NUM!" | -1.5707963267948966 |
| gen-m5/b0ecb7ca78460b77 Calc!A320 | `=SUMX2PY2(42,COUNT((Data!$C$8/Data!A4),ASIN((-1-2^-52))))` | 1765.0 | 1768.0 |
| gen-m5/c688ca172fe533dc Calc!A393 | `=ASIN((-1-2^-52))` | "#NUM!" | -1.5707963267948966 |
| gen-m5/d1b251fd6e9c4b4b Calc!A76 | `=ACOS((-1-2^-52))` | "#NUM!" | 3.141592653589793 |
| gen-m5/eb942f4a59b1c004 Calc!A56 | `=ASIN((-1-2^-52))` | "#NUM!" | -1.5707963267948966 |

## Acceptance criteria
- [ ] These 14 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ASIN accuracy inside [-1, 1].

## Outcome
`excel_asin` (ASIN, and ACOS through it) returns NaN, hence `#NUM!`, for `|x| > 1` before the kernel,
so `1 - x` rounding to 2 can no longer yield ±π/2. Unit test `asin_acos_just_below_minus_one_are_num`.
Eval delta: the 14 T-2170 known-failure lines left eval/baseline/gen-m5.txt, no new failures.
numbers.md "Just below -1 as well" now marks it implemented.
