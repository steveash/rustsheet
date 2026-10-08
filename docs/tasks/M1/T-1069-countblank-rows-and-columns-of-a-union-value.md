---
id: T-1069
title: "COUNTBLANK, ROWS and COLUMNS of a union: #VALUE!, #REF!, #REF! (SPEC-0004 row 5)"
milestone: M1
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
COUNTBLANK, ROWS and COLUMNS of a union give #VALUE!, #REF!, #REF!.

## Context
Oracle (probes/multiarea-count): COUNTBLANK((A1,B2))=#VALUE!, ROWS=#REF!, COLUMNS=#REF!. The engine gives 0, #VALUE!, #VALUE!. The engine's current outputs above are in eval/baseline/probes.txt; the fix needs T-1062's `Areas`/`RefArg` to tell a union from other arguments. Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] The three cells of multiarea-count pass
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).

## Outcome
`COUNTBLANK` of a union is `#VALUE!` (union goes through `evaluate_reference_arg`/`into_single_area`);
`ROWS`/`COLUMNS` of a union leaving several areas are `#REF!` (`rows_columns_extent`). Unit test
`countblank_rows_columns_of_a_union`. Eval delta: probes known failures 195 -> 192 (fixed 3:
multiarea-count H1:H3), 0 regressions. Cross-sheet unions in ROWS/COLUMNS stay `#VALUE!` (unprobed).
