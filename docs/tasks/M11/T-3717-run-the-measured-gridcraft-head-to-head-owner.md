---
id: T-3717
title: "Run the measured gridcraft head-to-head (owner request on issue #2277)"
milestone: M11
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Measure RustSheet against gridcraft on the same workbooks, as T-0496's note left for the owner.

## Context
The owner asked for the run in session on 2026-10-31. It means building and running gridcraft, so
it is owner-directed work, not issue work (decision 0158). Method: mog-head-to-head-2026-10-22.md.

## Acceptance criteria
- [x] docs/research/gridcraft-head-to-head-2026-10-31.md records values on all 17 suites,
      per-function rates, name recognition and load/recalc/save timings.
- [x] Results posted on issue #2277.
- [x] The temporary adapter is not merged.

## Out of scope
- Porting any gridcraft code (licence and provenance review first).

## Outcome
On the 3,773 cases both engines evaluated, RustSheet is 97.46% within 15 digits and gridcraft
83.39%, and 421 against 17 in-scope functions are at least 99.9% within 15 digits. The one lead is
gridcraft's FORECAST.ETS family. Eval delta: none (docs only).
