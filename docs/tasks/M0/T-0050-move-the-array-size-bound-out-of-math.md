---
id: T-0050
title: "Move the array-size bound out of math_and_trigonometry into a core limits module"
milestone: M0
lane: quality
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine-wide array-size bound (`MAX_SIZE`, `check_array_size`, `check_range_size`) lives in a core module, and `functions::math_and_trigonometry` is private again.

## Context
Raised by the architecture reviewer on T-0047. The bound lives in
`functions/math_and_trigonometry/array_size.rs`, but since T-0047 the core lift
`Model::evaluate_range` (cast.rs) uses it, so `functions/mod.rs` made `math_and_trigonometry`
`pub(crate)`. Move the file to a core location (e.g. `crates/rustsheet-engine/src/limits.rs`) and
update its importers (cast.rs, sequence.rs, random.rs, matrix.rs, bycol_byrow.rs). Pure move, no
behaviour change.

## Acceptance criteria
- [ ] The three items live in a core module; `mod math_and_trigonometry;` is private again; no other code changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
