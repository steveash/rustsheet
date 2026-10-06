---
id: T-0988
title: "Postfix % flushes subnormal results to 0"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The postfix `%` returns 0 for a subnormal result, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Subnormal results become 0); evidence probes/subnormal-function-results A1, A21 (T-0189). `OpUnary::Percentage` in model.rs does not call `arithmetic::finite_or_num`; reuse it.

## Acceptance criteria
- [ ] probes/subnormal-function-results A1 and A21 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
