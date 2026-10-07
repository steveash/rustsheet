---
id: T-0224
title: "LOOKUP with a 2-D lookup/result vector: probe Excel's error (engine returns #ERROR!)"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
LOOKUP with a 2-D lookup_vector or result_vector returns the error Excel returns (probe it); today it returns the non-Excel `#ERROR!`.

## Context
T-0210 made MATCH return `#N/A` for 2-D arrays and left LOOKUP unchanged (`as_vector(.., Error::ERROR)` in `lookup_and_reference/mod.rs`) because it is unprobed. Use the excel-probe skill, then record the rule in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] Also probe `MATCH(2,{1,2;3,4},0)` and `MATCH(2,A1:B2*1,0)`; mark the lookup.md array sub-rule VERIFIED or fix it.
- [ ] Probe evidence cited; LOOKUP on a 2-D vector matches Excel; unit test; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
