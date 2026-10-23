---
id: T-2421
title: "Probe and match BESSELK underflow to 0 near x 713 and BITLSHIFT shifts of 53 and -53 giving 0"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BESSELK and BITLSHIFT/BITRSHIFT give Excel's values where the smoke check found them differ.

## Context
Found by the T-2249 smoke check: a scratch suite of 790 cells, one formula per entry of the M9 engineering and database pools, filled through excel-graph on 2026-10-22 and deleted afterwards (docs/excel-semantics/engineering.md and criteria.md, "edge-pool smoke check (T-2249)"). Commit a probe case with these cells first (excel-probe skill).
- `BESSELK(713,1)` and `BESSELK(714,1)` are 0. Ours gives the subnormals 1.0465929954E-311 and 3.84750046922E-312. Find where Excel starts to flush: whether its result is cut at the normal range (2.2250738585072E-308) or its kernel underflows earlier. `BESSELI(713,0)` is `#NUM!`, which ours matches.
- `BITLSHIFT(1,53)` and `BITLSHIFT(1,-53)` are 0, while `BITLSHIFT(1,48)` and `BITLSHIFT(1,54)` are `#NUM!` and `BITLSHIFT(1,47)` is 140737488355328. Ours `#NUM!` at ±53. Probe shifts 48 to 54 and -48 to -54 with numbers 0, 1 and 2^47, and BITRSHIFT the same.
Code: functions/engineering (bessel_k.rs, the BIT* functions).

## Acceptance criteria
- [ ] A probe case holds the cells above and the shift grid, plus the BESSEL order-cap evidence the smoke check saw
      (`BESSELI(2.5,100001)` and `BESSELI(2.5,1E+6)` are 0) and `BESSELK(2.5,100001)`, so the DEVIATION entry
      "BESSEL order cap" cites a committed probe.
- [ ] Ours matches every probe cell; docs/excel-semantics/engineering.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BESSEL order cap (a DEVIATION, decision 0014).
