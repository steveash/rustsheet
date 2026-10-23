---
id: T-2172
title: "Oracle sweep: TAN on 1E+6 .. 2^27 and ODD/EVEN above 2^53"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph oracle sweep gives the fitting points T-2173 (ODD/EVEN above 2^53), T-2147 (TAN near 1.2E+8) and T-2384 (SUMXMY2 last digit) need.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Both functions show last-digit misses in gen-m5 that 2-3 cells cannot fit (docs/excel-semantics/numbers.md "ODD and EVEN above
2^53" and the TAN paragraph after "SIN/COS ... reduce every trig argument by multiples of pi64/2", both added by this triage):
`TAN(123456789.123)` is 56.41575748007382 (ours ...815), `ODD(9999999999999999)` is 9999999999999992 (ours ...988),
`EVEN(6.223015277861143E+37)` is 6.223015277861144E+37.
Sweep, as stored inputs (probes-numeric case, `excel-probe` skill):
- TAN, SIN and COS at ~150 arguments spread over 1E+6 .. 2^27-1 (log-spaced plus the two gen-m5 arguments 123456788.123 and
  123456789.123 and their neighbours), so T-2147 can tell whether SIN/COS also leave the pi64 model there.
  Add the fractional arguments just below the `#NUM!` cut, 134217727.25, 134217727.5, 134217727.75 and the
  largest double below 2^27 (134217728 - 2^-26), and their negatives: numbers.md "Trig of a huge angle" says
  only the integers 2^27-1 and 2^27 were probed, so the cut for fractions is assumed (gap triage 2026-10-20).
- ODD and EVEN at ~100 values from 2^52 to 1E+300: consecutive doubles around 2^53, odd and even integers near 1E+16,
  9999999999999990 .. 10000000000000010, and powers of ten; negatives of a few.
- SUMXMY2, SUMX2MY2 and SUMX2PY2 for T-2384 (gen-m5 round-2 triage, T-2252, 2026-10-22): `SUMXMY2(C10:E12,C6:E8)` in
  gen-m5/bf8903e05ec369ca Calc!A248, pairs (-1, 123456789.123), (7, 1E-15), (-1, 0.3), is 1.5241579027474522E+16, 1 ulp above the
  correctly rounded sum. About 60 cells: that cell's pairs in every order and alone, single pairs (x, y) with x - y near 1E+8,
  1E+9 and 2^27 and a small second pair, pairs whose squares cancel (x close to y, both large), and 2- to 5-pair sums of mixed
  magnitudes, each as the three functions.

## Acceptance criteria
- [x] A probes-numeric case with an excel-graph oracle sidecar covers both lists; numbers.md links it from both entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting (T-2173, T-2147, T-2384).

## Outcome
Added `eval/suites/probes-numeric/cases/trig-odd-even-sumxmy2-sweep` (case + excel-graph oracle sidecar): TAN/SIN/COS at 145
arguments (Calc!A1:D145), ODD/EVEN at 66 values (F1:H66), SUMXMY2/SUMX2MY2/SUMX2PY2 on 77 pair sets (J1:X77). Finding: the
trig cut for fractions is |x| >= 2^27 (134217728.25 is `#NUM!`, 2^27-2^-26 is a number), recorded in numbers.md. No eval
baseline change expected (new probe cells are oracle-only; checked by `make check`).
Pass counts of the new case (`probes-numeric` is `gate = false`, so its 20 failing cells are neither baselined nor gated; 778 of 798 pass):
TAN 4 misses (T-2147), SIN and COS none; ODD 11 misses, EVEN none (T-2173); SUMXMY2 5 misses, SUMX2MY2 and SUMX2PY2 none (T-2384).
