---
id: T-1317
title: "SUM/MAX: an error in a direct argument beats an error in a referenced cell (9 probes/sum-error-precedence cells)"
milestone: M2
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUM and MAX return the error of a direct argument (literal or computed) ahead of an error held in a referenced cell, in either order, as Excel does.

## Context
Rule and evidence: docs/excel-semantics/syntax.md "SUM and MAX: an error in a direct argument beats an error in a referenced cell" (probes/sum-error-precedence C1-C23). The engine takes the leftmost error; the 9 failing cells are C1, C3, C5, C7, C9, C11, C17, C19, C20. Among referenced cells the leftmost still wins (C13-C16 pass). Other aggregates (AVERAGE, MIN, PRODUCT, ...) are unprobed: probe them before generalizing, or limit the change to SUM and MAX. Also unblocks T-1314's C11.

## Acceptance criteria
- [ ] The 9 listed cells of probes/sum-error-precedence pass; baseline updated.
- [ ] Other aggregates are left as they are unless probed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
