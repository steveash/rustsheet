---
id: T-2175
title: "Catalog coverage counts volatile and oracle-none functions with the evidence floor as covered (decision 0085 item 4)"
milestone: M5
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
report.md's "Catalog coverage per milestone" table reads M5's coverage criterion directly, as
decision 0085 item 4 defines it. A new `covered` column counts the in-scope functions that meet
the criterion. A function meets it when it has (a) the evidence floor and (b) a generated cell,
or when it is volatile or `oracle: none` and has (a). The row is at 100% when `covered` equals
`functions`.

## Context
- Decision 0085 item 4 (docs/decisions/0085-how-m5-s-exit-criteria-are-measured.md) says volatile
  functions (RAND, RANDBETWEEN) and `oracle: none` functions (INFO, SHEETS) satisfy (b) without a
  generated cell and need (a) only. Today `crates/rustsheet-eval/src/catalog_coverage.rs`
  (`summarize`, T-2112) counts only `both`, so the M5 row reads 131 of 135 even though RAND and
  RANDBETWEEN already meet the criterion. The steward has to apply the exemption by hand.
- The tags already exist: `catalog.needs_other_evidence(&f.name)` (`oracle: none`) and
  `is_volatile_function(&f.name)` (crates/rustsheet-eval/src/volatile.rs) both fill
  `Uncovered::tags`.
- Decision 0085 also says M6's "same as M5" inherits these definitions, so the rule applies to
  every row, not only M5.
- Docs: docs/architecture/eval-harness.md has no catalog-coverage paragraph yet. Add one under
  "Milestone function scope" or next to it.

## Acceptance criteria
- [ ] `MilestoneCoverage` has a `covered: usize` field (report.json `catalog_coverage[].covered`).
      It counts (a && b) || (a && (volatile || oracle: none)). `functions`, `with_evidence`,
      `in_generated` and `both` keep their current meaning and values.
- [ ] The markdown table gains a `covered (decision 0085)` column. In each "Mk missing" list, a
      function that is covered under the exemption shows `exempt from b` and is listed apart from
      the functions that still miss the criterion. Either split the list in two, or drop the
      exempt functions from it and print a separate "Mk exempt from b" line.
- [ ] Unit tests in catalog_coverage.rs cover four cases: a volatile function with ≥ 20 cells and
      no generated cell (covered), a volatile function with fewer cells (not covered), an
      `oracle: none` function with ≥ 20 cells (covered), and an ordinary function with (a) but not
      (b) (not covered).
- [ ] On `make eval` at the PR's base, the M5 row's `covered` is 133 (131 plus RAND and
      RANDBETWEEN), or `both` plus whatever T-2115 has added by then. The PR text quotes the row.
- [ ] docs/architecture/eval-harness.md describes the section and the exemption, and cites
      decision 0085.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The evidence for INFO and SHEETS (T-2115).
- Removing `oracle: none` from the catalog.
- Any gate on the numbers. The table is an exit measure, not a gate.
- The combined exit section (T-2177).
