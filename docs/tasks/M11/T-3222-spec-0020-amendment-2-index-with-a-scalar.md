---
id: T-3222
title: "SPEC-0020 Amendment 2: INDEX with a scalar row as a consumer of a tailed argument (probes/t3096-lookup-value-blank-elements under 0.5 s)"
milestone: M11
lane: perf
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-3221]
---

## Goal
INDEX with scalar indices over a tailed argument picks its element from the head or the tail without building the
logical array, so probes/t3096-lookup-value-blank-elements runs in under 0.5 s (4.8 s today) with the same values.

## Context
Design: docs/specs/SPEC-0020-whole-column-array-operands.md "Amendment 2", section 3 "INDEX with scalar indices" (Rollout step 2). Needs T-3221's keyed lookups for the t3096
shapes (`INDEX(VLOOKUP(Keys!$A:$A,..),1000)`, `INDEX(MATCH(Keys!$A:$A,..,1),2)`).
Code: `functions/lookup_and_reference/index.rs` (`index_impl`, `index_source`, `index_from_array`), `tailed_array.rs`.

## Acceptance criteria
- [ ] The INDEX consumer of section 3, in `index_source`'s ordinary reading, with the fallback for every other index.
- [ ] The INDEX equivalence tests of the amendment's test plan.
- [ ] probes/t3096-lookup-value-blank-elements under 0.5 s through the harness (time before and after in the Outcome),
      and an `INDEX(MATCH(A:A,..),n)` bench line.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifted INDEX indices; IF and CHOOSE forms that `pick_reference` resolves.
