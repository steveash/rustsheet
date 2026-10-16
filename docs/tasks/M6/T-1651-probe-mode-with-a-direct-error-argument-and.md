---
id: T-1651
title: "Probe MODE with a direct error argument and LOOKUP with a scalar lookup_vector"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether MODE propagates or skips an error in a direct (non-reference) argument, and whether LOOKUP reads a scalar lookup_vector or array as a one-element array, with ours fixed if it differs.

## Context
T-1643 made MODE propagate the error of an argument picked by IF or CHOOSE (`MODE(CHOOSE(1,B1:B6*1))` off the range's rows is `#VALUE!`, probes/legacy-choose-arm-aggregate-and-array-constant-at I10) and LOOKUP's array form read a scalar picked by IF or CHOOSE as a one-element array (J1). Both rules are narrowed to IF/CHOOSE picks because nothing else is probed; any other direct error is still skipped by MODE and any other scalar lookup_vector is unchanged. Code: crates/rustsheet-engine/src/functions/statistical/mode_functions.rs (`collect_mode_values`), crates/rustsheet-engine/src/functions/lookup_and_reference/lookup.rs (`fn_lookup`). docs/excel-semantics/references.md "Legacy `@` on CHOOSE operator arms under aggregates" lists the open questions.
T-1291 answered the MODE half (probes/mode-typed-arguments A30-A33, probes/mode-error-and-computed-arguments: direct, referenced and named errors propagate in MODE, MODE.SNGL and MODE.MULT, and ours matches); `MODE(IF(TRUE,1/0),1,1)` and the LOOKUP and INDEX cells remain.

## Acceptance criteria
- [ ] A probe with oracle values for `MODE(1/0,1,1)`, `MODE(NA(),1,1)`, `MODE(#DIV/0!,"1")`, `MODE(A1)` and `MODE(nm)` with A1 and the name `nm` an error, MODE.SNGL/MODE.MULT likewise, `LOOKUP(2,1)`, `LOOKUP("b","a")`, `LOOKUP(2,1,5)`, `LOOKUP(2,IF(TRUE,1))`, `LOOKUP(2,{1})`, `MODE(IF(TRUE,1/0),1,1)` and `INDEX(IF(TRUE,5),1)` (IF picks are inferred from CHOOSE).
- [ ] references.md / coercion.md / lookup.md updated; ours fixed (and the IF/CHOOSE narrowing dropped) if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
