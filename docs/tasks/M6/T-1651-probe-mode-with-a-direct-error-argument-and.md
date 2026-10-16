---
id: T-1651
title: "Probe MODE with a direct error argument and LOOKUP with a scalar lookup_vector"
milestone: M6
lane: research
status: in-progress
priority: P3
size: S
tier: opus
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

## Outcome
Probe `probes/mode-direct-error-lookup-scalar-vector` filled from excel-graph: every MODE/MODE.SNGL/MODE.MULT error argument propagates (ours already did); LOOKUP (array and vector forms) reads any scalar lookup_vector as a one-element array, so the IF/CHOOSE narrowing in `fn_lookup` was dropped. references.md and lookup.md updated.

## Handoff (resumed)
Work is complete and all four reviewers (security, architecture, quality, excel-fidelity) APPROVE on PR #935; `make check` passed locally. The only red CI check is `hygiene` -> `scripts/check_governance.py`: "protected paths changed without a decision record: quality/evidence-floor.json". The new probe `probes/mode-direct-error-lookup-scalar-vector` lifts a function over the 20-cell evidence floor, so `rustsheet-eval evidence-floor` fails with "fewer functions are below the floor: run `make eval-baseline` and commit quality/evidence-floor.json" unless the lock goes 53 -> 52. The lock change is tool-generated (already on the branch), not hand-edited. A sonnet run may not add a decision record for a protected path (work-loop 5b), so next step: `make new-decision TITLE="..."` (or confirm an existing decision such as the evidence-floor ratchet already covers generated lock decreases), commit it, push, wait for CI green, then squash-merge once the branch is up to date with origin/main. Set status: done with the Outcome note once merged (flip status back from todo, tier back is not needed).
Note: probes/union-walk-repeated-in-lambda.case.json takes ~61s here against the 60s per-case default timeout, so local `make eval-baseline` records a `!Timeout` for it; regenerate with `rustsheet-eval update-baseline --timeout 180` (already done on the branch) instead of baselining the timeout.
