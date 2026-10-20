---
id: T-2134
title: "Fit Excel's SINH/TANH kernel on 0 .. 1 (COTH/CSCH inherit it)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0917, T-2178]
---

## Goal
SINH and TANH give Excel's exact doubles on |x| < 1 (where Excel is up to 2.6 ulp off the true
value), so COTH, CSCH and SECH, which are 1 over them (T-0917), follow.

## Context
T-0917 tried and rejected every simple formulation (docs/excel-semantics/numbers.md "SINH and TANH
on 0.4 .. 1: no formula found yet"): libm, exp forms with libm or the x87 EXP model V6, 80-bit
forms, Taylor series in several summation orders, `expm1` forms. Excel misses the true value by
more than half an ulp at e.g. SINH(0.01), 0.05, 0.43, 0.47, 0.65, 0.7, 0.88 and TANH(0.01),
0.54, 0.85, so some double-precision intermediate is involved. Ideas not yet tried: an x87 F2XM1
based `expm1` stored to a double then `(E + E/(E+1))/2` (fdlibm's shape) or `E*(E+2)/(2(E+1))`;
a fixed-degree minimax polynomial (fit lab, T-1523, docs/research/excel-kernel-playbook.md).
Fitting set: probes-numeric `m9-hyperbolic` A157-A248, `t0917-hyperbolic-switchover` A79-A210,
ironcalc-calc TRIGONOMETRIC.xlsx SINH_COSH_TANH. Needs an oracle sweep: T-2178 (research) fills a dense
SINH/TANH/COSH sweep (1035 points on (-1.5, 1.5)), `probes-numeric/t2178-hyperbolic-unit-sweep`; fit against it, not blind.
Known failures this owns: TRIGONOMETRIC.xlsx SINH_COSH_TANH!D10, D40, B51 and COTH_CSCH_SECH!B10,
B40, C51, C54, C55 (B10 and C54 were unmasked by T-0917, decision 0084), and POWER.xlsx
inside_a_lambda!C3/C4 (TANH in `gelu`, T-0959). Ungated but owned too: probes-numeric
`m9-hyperbolic` Calc!A126 (CSCH(0.001), passed on libm before T-0917) and `t0917-hyperbolic-switchover`
A81, A97, A105, A109, A159 (decision 0084), plus the other SINH/TANH/COTH/CSCH misses of both cases.

- gen-m5 (gap triage 2026-10-20, T-2114, excel-graph): 19 more cells, in eval/baseline/gen-m5.txt. All 19 are SINH, TANH, COTH or CSCH of an argument in (-1, 1) (the COTH/CSCH ones through 1/TANH and 1/SINH), so they belong to this kernel fit: they are fitting points, not acceptance cells, unless the fit makes them pass. E.g. `SINH(0.3)` Excel 0.3045202934471426, ours ...254; `SINH(PI()/4)` 0.8686709614860096 vs ...095; `TANH(0.2)` 0.19737532022490403 vs ...04.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/1b30e4d075b252ea Calc!A126 | `=SINH((0.3+Data!A6))` | 0.3045202934471426 | 0.30452029344714254 |
| gen-m5/1d22a002d4f380b3 Calc!A107 | `=CSCH(PI()/4)` | 1.1511838709208486 | 1.1511838709208488 |
| gen-m5/1d22a002d4f380b3 Calc!A391 | `=COTH(SIN((Data!E4*0.5)))` | -2.966743095697094 | -2.9667430956970935 |
| gen-m5/204b68ebb5812d50 Calc!A118 | `=COTH((Data!B9%*Data!A$9))` | 50.00666648889566 | 50.00666648889567 |
| gen-m5/2abdc1524e5aa685 Calc!A351 | `=SINH(PI()/4)` | 0.8686709614860096 | 0.8686709614860095 |
| gen-m5/399071ff6782ab16 Calc!A344 | `=SINH(PI()/4)` | 0.8686709614860096 | 0.8686709614860095 |
| gen-m5/461d1c8dbb044769 Calc!A80 | `=SINH((Data!B7+Data!C5))` | -1.0265167257081755 | -1.0265167257081753 |
| gen-m5/461d1c8dbb044769 Calc!A211 | `=SINH(PI()/4)` | 0.8686709614860096 | 0.8686709614860095 |
| gen-m5/4febabc66cfdf9a1 Calc!A247 | `=COTH((TAN(Data!E$2)*SIN(Data!$D8)))` | -19.699635232240993 | -19.699635232240997 |
| gen-m5/6361fdf5993da954 Calc!A17 | `=COTH(((Data!$B12*1e-15)+(Data!$D$1/Data!B2)))` | -2.631932441832191 | -2.6319324418321917 |
| gen-m5/8a387c26d182e73c Calc!A400 | `=CSCH(SUM(IF(Data!A$11,Data!A$11,0),Data!B7:B10,1))` | 3.2838533966984227 | 3.283853396698424 |
| gen-m5/90bf45bea539723d Calc!A130 | `=CSCH(PI()/4)` | 1.1511838709208486 | 1.1511838709208488 |
| gen-m5/9284730052aff46c Calc!A218 | `=SINH((Data!D7+ATANH(0.1)))` | 0.10050378152592127 | 0.10050378152592125 |
| gen-m5/aac6230c2fb40e1e Calc!A94 | `=SINH(PI()/4)` | 0.8686709614860096 | 0.8686709614860095 |
| gen-m5/aac6230c2fb40e1e Calc!A310 | `=COTH(Data!$A8%)` | 333.3343333327333 | 333.33433333273337 |
| gen-m5/cf893ec19f53a7fc Calc!A24 | `=TANH(Data!E9)` | 0.19737532022490403 | 0.197375320224904 |
| gen-m5/e0087a1fb9a36188 Calc!A134 | `=SINH(Data!E7)` | 0.3045202934471426 | 0.30452029344714254 |
| gen-m5/e0087a1fb9a36188 Calc!A265 | `=SINH(0.3)` | 0.3045202934471426 | 0.30452029344714254 |
| gen-m5/fc81c2f704bef138 Calc!A298 | `=SINH(0.3)` | 0.3045202934471426 | 0.30452029344714254 |

- Gap triage 2026-10-20 (after T-2114): 7 more TANH misses on (0, 1) outside the lists above, fitting points
  too: probes-numeric `power-ulp-sources` Calc!A4, A5, A6, A7, A11, A12 (`TANH(0.4)` Excel 0.3799489622552249,
  ours 0.37994896225522484; A5-A7 are `1+TANH(..)` and the gelu product over it) and
  gen-m6/ed0f4f783473cef0 Calc!A23 (`TANH(SUM(Data!D6:D11,(-0.0001)))` Excel 0.3798633951257878, ours ...879).

## Acceptance criteria
- [ ] COTH_CSCH_SECH!B10 and C54, m9-hyperbolic A126 and t0917-hyperbolic-switchover A81, A97, A105,
      A109, A159 pass (decision 0084); no new known failures.
- [ ] The fitted formulation and its evidence are in `docs/excel-semantics/numbers.md`, including
      negative results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SINH/COSH overflow near 710 (SINH(710) is `#NUM!` in Excel).
