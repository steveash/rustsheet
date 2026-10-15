---
id: T-1476
title: "Move MIN, MAX, PRODUCT, COUNT and COUNTA onto the shared aggregate-argument walker"
milestone: M2
lane: quality
status: in-progress
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0009-aggregate-argument-walker.md
depends_on: [T-1474]
---

## Goal
The last M2 aggregates with their own argument dispatch use the walker from T-1474. After this, "one implementation
of aggregate-argument iteration, used by every consumer" (M2 exit) holds for the consumers the spec names.

## Context
After T-1474, the per-argument dispatch with its own `reference_error` precedence is still in two places:
`Model::fold_min_max` (functions/math_and_trigonometry/mathematical.rs, MIN and MAX) and `Model::fn_product`
(functions/math_and_trigonometry/product.rs). COUNT and COUNTA (functions/statistical/count.rs) have a
dispatch of their own as well. The spec from T-1473 says how each one maps onto the walker's policy (for example,
PRODUCT's `seen_value` and MIN/MAX's "0 when there is none"). Evidence that must keep passing:
probes/min-max-value-args, probes/aggregate-error-precedence (C1-C24 cover PRODUCT, MIN and MAX) and the COUNT
probes named in the spec.

T-1474 built only the parts its consumers use. This task adds the rest of SPEC-0009's policy in
`functions/util/aggregate_args.rs`: `ErrorRule::Items` (COUNT, COUNTA), the `reading` field with
`ArgReading::PickedKeepingBlanks` (COUNTA) and `ArgReading::Reference` (PRODUCT), and the
`ReferenceValue` rules COUNT needs. `for_each_value` and its two siblings in count_and_average.rs
stay as named-policy entry points.

## Acceptance criteria
- [ ] MIN, MAX, PRODUCT, COUNT and COUNTA call the walker. `rg "let mut reference_error" crates/rustsheet-engine/src/functions` finds only the walker.
- [ ] `make eval`: 0 regressions and 0 fixes in every suite.
- [ ] scripts/check_duplication.py does not rise; record any drop with `--update` per decision 0010.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MINA/MAXA and the other A-variants unless the spec lists them; behavior changes of any kind.
