---
id: T-2918
title: "AMORDEGRC past 2^20 periods: weigh walking Excel's twice-rounded subtraction further before the closed form"
milestone: M9
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2849]
---

## Goal
AMORDEGRC gives Excel's value for more walks past 2^20 periods (probes/fin-amor-period-bound-c A1, A3, A4, A5 are known
failures), or financial.md "AMORDEGRC past 2^20 periods" records why the closed form stays, with the timings.

## Context
T-2849 found Excel's per-period arithmetic (financial.md "AMORDEGRC rounds its charge like ROUND"): the remainder is
rounded twice at each subtraction (`arithmetic::extended_sum`). A scratch walk of that arithmetic gives all of
fin-amor-period-bound-c A1-A5 (1.1E6 to 2.4E6 periods), where the closed form past `DEGRESSIVE_LOOP_PERIODS` (2^20,
functions/financial/amortization.rs) misses A1, A3, A4 and A5. A walked period costs about 7.4 ns in a release build
(T-2849: 512 elements of `AMORDEGRC(1E15,39679,39813,1,5E6,1E-7,1)`, 2^20 periods each, took 3.9 s), so raising the
bound to 2^22 would cost about 31 ms a scalar call and would need `AMORDEGRC_WEIGHT` (lift/kinds/financial.rs) raised
to match, which lowers the lifted element count below 512. Weigh that trade (SPEC-0011 security rule: about 5 s a
call at the cap) against the cells it fixes; a life is `ceil(1/rate)`, so a walk stays unbounded without some cap.

## Acceptance criteria
- [x] Either the bound rises (cells fixed, baseline lines removed, weight refitted with timings in the PR and in
      financial.md "AMORLINC and AMORDEGRC work cap") or the entry records the timings and why the bound stays.
- [x] No baseline gains a line (`make eval-check`: 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- The arithmetic itself (T-2849).

## Outcome
The bound rises: `DEGRESSIVE_LOOP_PERIODS` 2^20 -> 2^22 periods and `AMORDEGRC_WEIGHT` 2^18 -> 2^20 units, so a lifted
call runs at most 128 elements (was 512). Timings in a release build (4 cores, load about 1): 128 elements of
`AMORDEGRC(1E15,39679,39813,1,5E6,1E-7,1)`, a full 2^22-period walk each, took 3.96 to 4.35 s over five runs (about
7.6 ns a period, 32 ms an element); the scalar call about 28 ms. probes/fin-amor-period-bound-c A1, A3, A4 and A5 are
now Excel's (all six cells pass), recorded in financial.md "AMORDEGRC past 2^22 periods" (renamed) and "AMORLINC and
AMORDEGRC work cap", and SPEC-0011's as-built note. T-2907 (pin the same four cells) is dropped as done by this.
Eval delta: -4 known failures (probes), 0 regressions.
