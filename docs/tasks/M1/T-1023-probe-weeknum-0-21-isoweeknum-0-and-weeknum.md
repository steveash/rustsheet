---
id: T-1023
title: "Probe WEEKNUM(0,21), ISOWEEKNUM(0) and WEEKNUM error precedence at serial 0 (return_type 0/3)"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle values for WEEKNUM(0,21), ISOWEEKNUM(0), WEEKNUM(0,0), WEEKNUM(0,3) and WEEKNUM(0,99), and the engine matches them.

## Context
T-0995 made WEEKNUM accept serial 0 for return types 1, 2, 11-17 (`functions/date_and_time/weeknum.rs`). Serial 0 with type 21 still gives #NUM! and the error precedence of an invalid return_type at serial 0 changed (return_type error now first); neither is probed.

## Acceptance criteria
- [ ] Probe case in eval/suites/probes with oracle values; dates.md updated; engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done by the sonnet run: probe `probes/weeknum-serial-0` (94 cells, oracle-filled), findings in dates.md ("WEEKNUM return_type edge cases"), baseline updated (new-coverage failures only).
Not done (beyond sonnet: unclear evidence): the engine differs from the oracle on `WEEKNUM(0,21)`/`ISOWEEKNUM(0)` (52; engine `#NUM!`), invalid return_type error kind (`#NUM!` for 0, 3-10, 18-20; engine `#VALUE!` for <=0 and 3),
return_type >= 22 / negative (acts like type 2; engine errors), boolean return_type (`#VALUE!`; engine accepts). The cell `WEEKNUM(0,99)` is 0 in C5 but 1 in C68 (same workbook, reproducible), which I could not explain.
Next: re-probe the anomaly in isolation (single-cell workbooks, desktop backend if available), decide the serial-0 rule, then fix `fn_weeknum` and `fn_isoweeknum` (isoweeknum goes through `from_excel_date`, which rejects serial 0).
