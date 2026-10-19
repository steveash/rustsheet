---
id: T-1860
title: "Probe CONVERT temperature sweep, prefixed temperature units and prefix overflow"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CONVERT's temperature forms, prefixed temperature units and prefix overflow match Excel on oracle evidence.

## Context
Found by T-1297 review. `convert_temperature` in crates/rustsheet-engine/src/functions/engineering/convert.rs has per-pair forms fitted to two values each (68, -40.5). Unprobed: `CONVERT(1E290,"Ym","Ym")` (the prefix multiply can overflow), a truly overflowing result, and prefixed temperature units such as "mK".

## Acceptance criteria
- [ ] Probe case sweeping small, large and fractional values for every temperature pair, `mK`-style prefixes and prefix-overflow cases; oracle values recorded.
- [ ] The engine matches them; findings in docs/excel-semantics/engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes: convert-temperature-sweep (140 cells), -prefix-a/-b (176), -ov00..ov12 (single cells).
`convert_temperature` is now one formula per pair, matching all 9 oracle points per pair; the
prefix path combines the prefixes when the from-prefix multiply would overflow (`Ym`,`Ym` at
1E290/1E300). Prefixed temperature units: only K/kel take prefixes, already handled. Not probed:
`CONVERT(1E308,...)` temperature overflow (excel-graph rejects 1E308 workbooks); see
docs/excel-semantics/engineering.md.
