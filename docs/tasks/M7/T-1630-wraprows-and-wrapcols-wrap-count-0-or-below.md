---
id: T-1630
title: "WRAPROWS and WRAPCOLS: wrap_count 0 or below is #NUM!, above the vector length it clamps, and a scalar is a 1x1 result"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WRAPROWS and WRAPCOLS match Excel for a wrap_count of 0 or below (`#NUM!`), above the vector's length (clamped to it) and a scalar argument (a 1x1 result).

## Context
Found by T-1627: probes/dynamic-array-reshape-and-isomitted cells C20, C21, C32-C37, C46 and D20, D21, D32-D37, D46 (docs/excel-semantics/arrays.md, "WRAPROWS and WRAPCOLS"). Ours gives `#VALUE!` for 0 and -1, pads past the vector (1x8, 1x100) and `#N/A` for a scalar.

## Acceptance criteria
- [ ] The 18 failing cells of probes/dynamic-array-reshape-and-isomitted pass; baseline updated with `make eval-baseline`
- [ ] `make check` green; reviewers approve.

## Out of scope
-
