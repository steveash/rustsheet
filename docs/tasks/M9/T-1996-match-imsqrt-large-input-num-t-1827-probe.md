---
id: T-1996
title: "Match IMSQRT large-input #NUM! (T-1827 probe A33, A34)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSQRT of a large real input gives what Excel gives: `#NUM!` for `IMSQRT("1E307")` and `IMSQRT("4E307")`.

## Context
Known failures probes/eng-im-overflow S!A33, S!A34 (oracle `#NUM!`; ours returns text). Find the threshold with a probe first (excel-probe skill), then implement in `imsqrt` (functions/engineering/complex.rs). See docs/excel-semantics/engineering.md "IM* results that overflow".

## Acceptance criteria
- [x] A33 and A34 pass and the threshold is pinned by a probe; no finite IMSQRT result changes.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
IMSQRT squares each part: a part past sqrt(DBL_MAX) (1.34078E154) is `#NUM!`; when only the sum of squares overflows the result is `0`. Pinned by probes eng-imsqrt-large and eng-imsqrt-large-b (excel-graph). Eval: probes known failures S!A33, S!A34 fixed; no finite result changed.
