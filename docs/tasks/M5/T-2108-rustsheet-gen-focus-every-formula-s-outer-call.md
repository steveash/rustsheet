---
id: T-2108
title: "rustsheet-gen --focus: every formula's outer call is a function of the target milestone, cycled evenly"
milestone: M5
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-gen --milestone M5 --focus` writes cases where every formula's outermost call is a
generator-ready function whose catalog milestone is exactly M5. The functions take turns through a
seeded shuffle, so a run of N formulas gives each one about the same number of outer calls.
Arguments are still drawn from everything in scope at or below the milestone. Without `--focus`
the output is byte-identical to today.

## Context
- M5 exit (docs/milestones.md) needs "bit-exact ≥ 99.9% on generated suites (≥ 20k cells)" for
  the math, logical, text and information functions. Today `Catalog.scoped("M5")`
  (tools/src/rustsheet_tools/gen/generator.py) keeps every function with milestone ≤ M5, and
  `Gen.weighted` picks by sqrt(prior). The high-prior M2 functions crowd out the M5 ones. In the
  committed eval/suites/gen-m5 (10 cases, 400 formulas), M2 functions make 451 of the 616 calls
  and are the outer call of 132 formulas, against 41 for M5. Only 61 of the 131 generator-ready
  M5 functions (those with `params` and not `volatile`) appear at all.
- Design: docs/architecture/synthetic-generator.md §5.4 (milestone scope). The focus mask is a
  generation option and needs no catalog change.
- Implementation hint (not binding): in `generate_case`, when focusing, build the list of
  `scoped(milestone)` functions whose own `milestone` equals the target. Shuffle it with the
  case's `rng`, then take formula i's outer function from position `(case_index * formulas + i) %
  len`, so the turn continues across cases. A new `Gen` method can emit a call to a given function
  (refactor `Gen.call` so the pick and the argument building are separate; do not copy the
  argument loop). `--focus` then skips the formula's random `typ` draw.
- The committed gen-m5 cases no longer regenerate byte-identically from seed 1, because the catalog
  has changed since T-1192. So "unchanged without the flag" is checked against a digest recorded
  before the change, not against eval/suites/gen-m5.

## Acceptance criteria
- [ ] `rustsheet-gen --milestone M5 --focus --count 50 --formulas 400 --seed 2 --suite <scratch>`
      gives each of the 131 generator-ready, non-volatile M5 functions at least 100 outer calls
      (a tools test asserts this on `generate_case` output, without writing files).
- [ ] With `--focus`, no formula's outer call is a function from another milestone (tools test).
- [ ] Without `--focus`, `generate_case` output is unchanged: a tools test pins the SHA-256 of
      `generate_case(catalog, 7, milestone="M5")` on today's catalog, recorded before the change.
- [ ] `--focus` with a milestone that has no generator-ready functions exits with a clear error,
      not a traceback or an empty suite.
- [ ] docs/architecture/synthetic-generator.md §5.4 documents `--focus`.
- [ ] `make check` green (ruff, mypy strict, tools tests); reviewers approve.

## Out of scope
- Generating or oracle-filling gen-m5 (T-2114).
- Edge pools (T-2109, T-2110).
- Changing the weighting of the inner arguments.
