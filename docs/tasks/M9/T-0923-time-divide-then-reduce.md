---
id: T-0923
title: "TIME: divide by 86400 first, then drop whole days (7 known failures)"
milestone: M9
lane: functions
status: done
priority: P2
size: S
depends_on: []
---

## Goal
`TIME(h, m, s)` reproduces Excel's floating-point result when the total exceeds 24 hours.

## Context
Oracle-free evidence (Excel-cached):
- `ironcalc-calc/DATE_AND_TIME/TIME_HOUR_MINUTE_SECOND.xlsx`: `Sheet1!D3` `=TIME(24,2,3)` expected 0.0014236111111110006 (we give 0.0014236111111111112);
  `D14` `=TIME(34,1,2)` expected 0.41738425925925915 (we 0.41738425925925926); `D17` `=TIME(32767,32767,32767)` expected 0.4257754629629744 (we 0.425775462962963); `E3/E14/E17` are the same values by reference. That is 6 lines.
- `ironcalc-calc/DATE_AND_TIME/DATE.xlsx` `Sheet1!D4` `=TIME(23,120,0)` expected 0.04166666666666674 (we 0.041666666666666664).

Root cause: `fn_time` in `crates/rustsheet-engine/src/functions/date_and_time.rs` reduces total seconds with `rem_euclid(86400)` before dividing.
Excel divides first and then drops the whole days in floating point. Checked offline: `((h*3600+m*60+s)/86400) % 1.0` reproduces
all 7 expected values and the already-passing TIME cells (3,2,1 / 16,60,24 / 10,61,2 / 10,1,62).
Keep the existing floor-of-arguments and negative-total (#NUM!) rules.

## Acceptance criteria
- [ ] The 7 `F` lines above disappear from `eval/baseline/ironcalc-calc.txt` with no new ones.
- [ ] Unit test covering TIME(24,2,3) and TIME(34,1,2) exact bits; semantics note in `docs/excel-semantics/` (date-time section).
- [ ] `make check` green.

## Out of scope
TIMEVALUE fractional seconds (T-0910, oracle-blocked).

## Outcome
TIME now computes `(total/86400) % 1.0`; the 7 known failures are fixed (ironcalc-calc 70046 -> 70053 cells, 0 regressions).
