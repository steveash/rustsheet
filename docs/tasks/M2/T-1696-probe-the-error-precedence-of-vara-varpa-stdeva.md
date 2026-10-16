---
id: T-1696
title: "Probe the error precedence of VARA, VARPA, STDEVA and STDEVPA (direct vs referenced and array errors)"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The error order of VARA, VARPA, STDEVA and STDEVPA has excel-graph evidence, and the engine matches it.

## Context
Found in T-1439 review. `VARA_ARGS` (`functions/statistical/variance.rs`) uses
`ErrorRule::First`: the first error in argument order returns at once. The SUM, AVERAGE and
DEVSQ families use `Ranked` instead, where a direct error beats referenced and array errors
(probes/aggregate-error-precedence-stat-family, T-1439). Probe the same eight shapes with the
same data as that case, for the four A-variance functions. If they rank like the rest, switch
`VARA_ARGS` to `Ranked`. docs/excel-semantics/syntax.md, "Aggregate error precedence".

## Acceptance criteria
- [ ] A probe case (4 functions × 8 shapes) is oracle-filled, and the engine matches it.
- [ ] The UNVERIFIED note on VARA in syntax.md is resolved.
- [ ] The same case adds `fn(A7:A8,#N/A)` for SUM, AVERAGE, MEDIAN and DEVSQ too. T-1439's
      `fn(A7:A8,#DIV/0!)` cannot tell "a direct error wins" from "the range's first error wins",
      because A7 is `#DIV/0!` already.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed numeric text (T-1291).

## Outcome
Probe `probes/aggregate-error-precedence-a-variants` filled from excel-graph (72 cells): VARA, VARPA, STDEVA and STDEVPA rank errors like SUM, and `fn(A7:A8,#N/A)` is `#N/A` for SUM, AVERAGE, MEDIAN and DEVSQ. `VARA_ARGS` switched from `First` to `Ranked` (12 cells fixed); syntax.md updated.
