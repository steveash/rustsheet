---
id: T-2172
title: "Oracle sweep: TAN on 1E+6 .. 2^27 and ODD/EVEN above 2^53"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph oracle sweep gives the fitting points T-2173 (ODD/EVEN above 2^53) and T-2147 (TAN near 1.2E+8) need.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Both functions show last-digit misses in gen-m5 that 2-3 cells cannot fit (docs/excel-semantics/numbers.md "ODD and EVEN above
2^53" and the TAN paragraph after "SIN/COS ... reduce every trig argument by multiples of pi64/2", both added by this triage):
`TAN(123456789.123)` is 56.41575748007382 (ours ...815), `ODD(9999999999999999)` is 9999999999999992 (ours ...988),
`EVEN(6.223015277861143E+37)` is 6.223015277861144E+37.
Sweep, as stored inputs (probes-numeric case, `excel-probe` skill):
- TAN, SIN and COS at ~150 arguments spread over 1E+6 .. 2^27-1 (log-spaced plus the two gen-m5 arguments 123456788.123 and
  123456789.123 and their neighbours), so T-2147 can tell whether SIN/COS also leave the pi64 model there.
- ODD and EVEN at ~100 values from 2^52 to 1E+300: consecutive doubles around 2^53, odd and even integers near 1E+16,
  9999999999999990 .. 10000000000000010, and powers of ten; negatives of a few.

## Acceptance criteria
- [ ] A probes-numeric case with an excel-graph oracle sidecar covers both lists; numbers.md links it from both entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting (T-2173, T-2147).
