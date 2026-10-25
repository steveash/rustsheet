---
id: T-2385
title: "Probe [h]:mm and [m] without seconds on near-midnight serials (T-1732 follow-up)"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior is evidenced for `[h]:mm` and `[m]` alone on values within a second of an hour/minute boundary.

## Context
T-1732 rounds an elapsed count to the second only when a seconds token is shown; `[h]`/`[m]` without one keep their ms-snap truncation, unprobed (`DisplayClock` in formatter/elapsed.rs). Use the excel-probe skill.

## Acceptance criteria
- [x] Probe cases for `[h]`, `[h]:mm`, `[m]` on e.g. (3-0.4/3600)/24; the engine matches or a fix is filed; number-formats.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/elapsed-no-seconds-boundary, 32 cells, excel-graph): `[h]`, `[h]:mm` and `[m]` without a seconds token round the
serial to the whole second like every other date token (0.4 s short of 3 h is "3", "3:00", "180"; 0.6 s short "2", "2:59",
"179"). The engine truncated them (and `[h]:mm` printed "2:00"); `DisplayClock` now always counts the elapsed tokens from the
rounded serial. Two new quirks filed: `[h]:mm AM/PM` is #VALUE! (T-2683) and `[h]:mm.0` shows the seconds' tenths (T-2684).
number-formats.md updated. Eval delta: probes +1 case (30 of 32 cells pass, A31/A32 new-coverage failures owned by T-2683/T-2684),
0 regressions in any suite (poi included).
