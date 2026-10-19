---
id: T-1996
title: "Match IMSQRT large-input #NUM! (T-1827 probe A33, A34)"
milestone: M9
lane: functions-engineering
status: todo
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
- [ ] A33 and A34 pass and the threshold is pinned by a probe; no finite IMSQRT result changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
