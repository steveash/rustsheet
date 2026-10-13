---
id: T-1503
title: "Probe LET binding of @INDEX(...): does ISREF see a reference?"
milestone: M1
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`LET(x,@INDEX(B1:B2,2),ISREF(x))` matches Excel, by a probe case, and `bind_argument` follows the evidence.

## Context
T-1223 made `bind_argument` (functions/logical/let.rs) use `Model::eval_ref_or_index`, whose `@INDEX` arm keeps the reference (T-1247, spill anchors). A LET binding of `@INDEX(..)` was a value before (ISREF FALSE, ROW #VALUE!) and T-1223 kept that, as no probe says what Excel does. Probe it with excel-probe (also `LAMBDA` argument binding) and drop the exclusion in `bind_argument` if Excel keeps the reference.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers ISREF, ROW and the value of a LET and a LAMBDA binding of `@INDEX`; the implementation matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any other LET binding form.
