---
id: T-1841
title: "AMORDEGRC and AMORLINC bound their period loops, then lift over arrays (SPEC-0011)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1822]
---

## Goal
AMORDEGRC and AMORLINC lift over arrays in every argument and refuse a multi-cell range there, without a call able to loop for an unbounded time.

## Context
Split from T-1822. `amor_linc` and `amor_degrc` (functions/financial/financial_depreciation.rs) loop `while counted <= period`,
and `period` is only bounded by `ceil(1/rate)`: a tiny rate and a period of 1E15 loop 10^15 times today (directive 5).
Excel gives 0 for `AMORLINC(2400,39679,39813,300,1E9,0.15,1)` and `AMORDEGRC(...)` (probes/fin-remaining-later-argument-ranges
K4, K5), where the period is past the asset life. Bound the loops (the depreciation is constant or 0 once the
available amount is used up, so the loop can stop early; or a cap with a DEVIATION entry), then enter both as
`LiftArrays`: every argument is probed over an array (fin-argument-modes H19-H35) and over a range (H20, H29;
fin-remaining-later-argument-ranges H11-H34, all `#VALUE!`).

## Acceptance criteria
- [x] `AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` finishes in milliseconds; any capped result is a DEVIATION.
- [x] The AMORDEGRC and AMORLINC lift cells of probes/fin-argument-modes pass (baseline lines removed), no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AMOR* base values (probes/fin-amordegrc).

## Outcome
- Probed first: probes/fin-amor-period-bound (31 cells) and fin-amor-period-bound-b (5 cells; excel-graph,
  2026-10-18). Excel walks the periods one by one in doubles: the AMORLINC clamp period at rate 1E-6 is the
  remainder of 874,999 subtractions (A6, 0.0015212859222804571), not the closed form; it finishes 10^9 periods (-b
  A1) and AMORDEGRC 5E8 (-b B1), but a workbook holding only `AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` timed out
  (8.5 minutes, createSession retries exhausted), as did twice a larger one with periods 1E12 to 1E308; neither is in
  the suite (financial.md "AMORLINC and AMORDEGRC over many periods").
- functions/financial/amortization.rs (moved out of financial_depreciation.rs): one kernel, `amortization_of`, for
  both, its checks written as valid ranges so a NaN fails them, and the walks on an integer count:
  - AMORLINC (`linear_charge`) is exact for any period: within a binade the subtraction rounds the charge the same
    way each time, so after two equal steps the next ones are jumped in one exact multiplication, and a charge below
    half an ulp stops the walk for ever. At most 616 single steps over every charge from 2^-1074 to 2^1023 and values
    from 1E-300 to `f64::MAX` (cap 4,096, tested); bit for bit the plain walk in 2,820 seeded cases, ties included.
  - AMORDEGRC (`DegressiveWalk::charge`) walks up to 2^20 periods with main's arithmetic, stopping early below the
    salvage, at a charge that rounds to 0 before the half period, or at a fixed point (jumped to the half period);
    past 2^20 a closed form `remain * exp(n * log1p(-rate))`, which gave the walk's rounded value in all 2,700 seeded
    cases (466 past the cut with a charge of 1 or more) and Excel's -b B1.
  - `AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` takes 17.5 us in release (main: 10^15 steps; past 2^53 main's
    `counted += 1.0` never ended). Where Excel does not finish, the kernel returns the walk's value (2.4E-297 here), a
    DEVIATION.
- `LiftArrays` and `Weighted` (kinds.rs, `// --- Financial ATP (T-1841) ---`), weights from the derived worst case
  measured in release: AMORLINC 2^9 (its longest real walk, `AMORLINC(..,9E15,1.08E-16,1)`, crosses the most binades:
  2^18 elements in 1.23 s, 4.7 us each), AMORDEGRC 2^18 (2^20 walked periods with no shortcut,
  `AMORDEGRC(1E15,..,1,5E6,1E-7,1)`: 512 elements in 2.06 s, 4 ms each; a walk of 10^6 periods to the half period
  2.21 s). The per-period `amor_round` check is guarded by a `< 0.5` compare (4.5 s → 2.06 s). Both sides of each cap
  tested (262,144 / 512 elements run, one more is `#ERROR!`): financial.md "AMORLINC and AMORDEGRC work cap", a
  DEVIATION.
- Eval: probes 26 known failures fixed (fin-argument-modes H19, H21-H26, H28, H30-H35;
  fin-remaining-later-argument-ranges H11-H33 odd rows), 0 regressions. probes and probes-numeric compared cell by
  cell against main: no other cell changed. The new cases add 2 known failures, base values that main has too
  (fin-amor-period-bound B17, period 0.5, Excel 485, ours 0; -b B2, a cost of 1E308, Excel 1.94700443465919E302,
  ours 0), listed in T-1895. No scalar value changed where main finished within 2^20 AMORDEGRC periods; past that the
  closed form replaces main's walk (equal on every case tried).
- Ratchets: duplicated lines 4152 → 4066 (the shared kernel; the lift tests' `check_cells` and `round_trip_value` now
  in test/util.rs, also used by test_fn_cumulative_lift.rs), static_analysis.rs 1170 → 1166 (decision 0010).
- Follow-ups: T-1895 (AMORDEGRC below period 1 and at a cost of 1E308).

