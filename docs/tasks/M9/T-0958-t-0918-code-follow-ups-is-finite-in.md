---
id: T-0958
title: "T-0918 code follow-ups: is_finite in rept_element, fold the two R1C1 reference parsers"
milestone: M9
lane: quality
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`rept_element` checks its count is finite explicitly, and one R1C1 reference parser serves both callers.

## Context
T-0918 review follow-up (carried in T-0920): `rept_element` in
crates/rustsheet-engine/src/functions/text/common.rs relies on implicit range checks for a NaN or
infinite count; `parse_reference_r1c1` in expressions/utils/mod.rs and the R1C1 parser added for
INDIRECT duplicate logic (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] Explicit `is_finite` check with a unit test; one R1C1 parser, no behavior change (eval unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done on this branch: explicit `is_finite` check in `rept_element` plus a unit test
(`rept_tests` in functions/text/common.rs).

Not done: folding `parse_reference_r1c1` (expressions/utils/mod.rs, used only as an
"is this name R1C1-shaped" test in `is_valid_a1_identifier` and `name_needs_quoting`) into
`ParsedReference::parse_r1c1_reference` (utils.rs). They are not equivalent: the old one returns
`None` for len < 4 (so `RC`, `R1`, `R1C` are not seen as R1C1), rejects bare `R`/`C`, ignores
sheet bounds, and returns a different `ParsedReference` struct (relative offsets, no cell context);
the new one accepts `R`, `C`, `RC`, wraps offsets and range-checks. Replacing it changes which
names are valid/quoted, so it needs oracle evidence (excel-probe) for names like `RC`, `R1`, `R1C`,
`R0C1`, `R[1]C[1]` as defined names and sheet names before choosing the surviving semantics, and
`DEVIATION`/eval baseline care. Beyond a no-behavior-change S task for sonnet.
