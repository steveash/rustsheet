---
id: T-3795
title: "IMSQRT, IMLOG10, IMLOG2 and IMSECH last digits on the T-3643 probes: 31 cells one 15-digit unit off"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The 31 cells of the T-3643 probes where the engine prints one digit off Excel in the 15th print Excel's digits, from a
stated kernel model rather than per-point patches.

## Context
Follow-up of T-3643 (docs/excel-semantics/engineering.md, the T-3643 entries under the IMLN/IMLOG modulus). The cells:
- IMSQRT, 14 cells (probes/t3643-imsqrt-grid A5, A15 and the 1E-300, 1E-20, 1E-8 imaginary columns B, C, D, L, M, N of
  rows 5 and 15): a part of 0.999999999999999 (1-9·2^-53). Excel prints `1` / `+1i`, so its root is at least
  1-4·2^-53, where the engine's `sqrt(sqrt(x²+y²))` gives 1-5·2^-53. Neither `pow(·, 0.25)` nor a correctly rounded
  `exp(0.25·ln ·)` reaches 1-4·2^-53 for the real input; an extended-precision or doubly rounded root chain is the
  first lead (`arithmetic::extended_sqrt`).
- IMLOG10, 2 cells (t3643-imlog10-grid I8, S8, `10±1E8i`): imaginary part `atan2/LN_10` prints 0.682188133491472,
  Excel ...473.
- IMSECH, 4 cells (t3643-imsech-grid H5, H15, R5, R15, `±0.999999999999999±10i`): the imaginary part one digit off.
- The T-3643 fresh sweep (probes-numeric/t3643-r4-log-modulus), 11 cells: B29, B52 (IMLOG10's real `log10 |z|`) and
  B56, B99, B101, B132, B144, B167 (IMLOG10's imaginary `atan2/LN_10`), C66, C111, C173 (IMLOG2's imaginary `atan2·K`).

## Acceptance criteria
- [ ] Each cell matches Excel, or the task records a time-boxed negative fit (decision 0056) naming what was tried.
- [ ] No other IMSQRT, IMLOG10, IMLOG2 or IMSECH cell of the eval suites regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSECH's `#NUM!` for |y| ≥ 1E20 (T-3642).
