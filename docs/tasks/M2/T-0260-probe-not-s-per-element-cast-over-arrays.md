---
id: T-0260
title: "Probe NOT's per-element cast over arrays: text, empty and error elements"
milestone: M2
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Probe what Excel's NOT returns per element for text, empty and error elements of a range/array, and make the engine match.

## Context
T-0225 lifted NOT over arrays (shape/spill is oracle-backed) but the per-element cast is assumed to equal the scalar `cast_to_bool`: text "TRUE"/"false" -> boolean, other text -> #VALUE!, empty -> TRUE, error stays. Probe `NOT({"TRUE","abc","1",#N/A,0,2})`, `NOT({""})`, and NOT over a range holding text "TRUE", text "abc", an empty cell and an error. Update docs/excel-semantics/arrays.md.

## Acceptance criteria
- [x] Probe evidence cited; engine matches; probes-suite case; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed with probes/not-array-cast (38 cells, excel-graph 2026-10-09): NOT casts each element of a range or array
exactly like a scalar argument (text "TRUE"/"false" are booleans, other text including "" is `#VALUE!`, an empty
cell reads FALSE, errors stay). The engine already matched all 38 cells, so only the doc comment changed; arrays.md's
NOT status moves from ASSUMED to VERIFIED. Eval delta: probes +1 case, 38/38 pass; fixed 0, regressions 0.
