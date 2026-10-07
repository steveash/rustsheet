---
id: T-1027
title: "Addition and subtraction keep subnormal results; decide how to score kept subnormals"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`3E-308-2.5E-308` keeps its subnormal result as in Excel, and the oracle-unscoreable cells are handled by a documented scoring decision.

## Context
Left over from T-0989. probes/subnormal-function-results C69, C73-C75 (and A4, A6, A60-A62, A65) still fail: `arithmetic::plus`/`minus` flush via `finite_or_num` but Excel keeps a subnormal from `+`/`-` (numbers.md "Subnormal results become 0"). Trying it regressed A69, A73-A75 (Graph reports 0.0 for a kept subnormal, so the numeric cells expect 0) and probes/text-number-tiny B3,B5-B8,B18,B20 and text-number-words H6,H7,K1,K4,K5 (`"1e-308"+0` must be 0: flush subnormal text in the number-text cast first). Needs a scoring decision (decision record: protected baseline paths) for cells the oracle cannot score.

## Acceptance criteria
- [ ] C69, C73-C75 pass with no regression in text-number-tiny/words.
- [ ] Decision record on scoring A4, A6, A60-A62, A65, A69, A73-A75; no hand-edited baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
