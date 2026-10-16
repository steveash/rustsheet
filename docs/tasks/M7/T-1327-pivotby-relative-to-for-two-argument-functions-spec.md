---
id: T-1327
title: "PIVOTBY relative_to for two-argument functions (SPEC-0006 step 6)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1326, T-1325]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
PIVOTBY's relative_to 0-4 gives two-argument functions such as PERCENTOF the probed reference sets.

## Context
SPEC-0006 relative_to bullet; anchors P!Y25, P!A49, P!M49, P!Y49, P!A73. With several row or column levels, 3 and 4 are parent totals: probe a two-level case before implementing that part.

From T-1322 (groupby.rs): `aggregate` and `total_row` call the function with one argument; they need a relative-set
parameter and an arity choice (SPEC-0006 `Aggregate { rows, relative }`). GROUPBY with `_xleta.PERCENTOF` still calls it with one argument
(the values are wrong, the key column matches Excel) until T-1325 adds the relative set.

## Acceptance criteria
- [ ] Those anchors pass and leave the baseline; the two-level parent totals are probed and recorded in arrays.md.
- [ ] Relative sets are built once and shared; built-in PERCENTOF is O(N); a two-parameter LAMBDA is charged against the cell-visit cap, with a DEVIATION entry and test (SPEC-0006 Design step 3).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
