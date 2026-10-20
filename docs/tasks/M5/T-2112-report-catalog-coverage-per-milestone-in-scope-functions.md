---
id: T-2112
title: "Report catalog coverage per milestone: in-scope functions with Excel evidence and with cells in the milestone's generated suite"
milestone: M5
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`make eval` has a "Catalog coverage per milestone" section. For each function milestone (M2, M5, M6,
M7, M9, M10), it lists the in-scope catalog functions of that milestone (the catalog's own
`milestone`, not cumulative) and counts how many meet two conditions: (a) at least 20 Excel cells,
the evidence floor (decision 0062), and (b) at least one Excel-backed scored cell, in a generated
suite tagged with that milestone, whose formula calls the function. Each function that fails (a) or
(b) is listed by name, tagged `oracle: none` or `volatile` where that applies. The M5 exit criterion
"catalog coverage 100% for these categories" becomes a number.

## Context
- Today the report's "Excel evidence per function" section is not split by milestone. It has the
  evidence counts, the `oracle: none` list (INFO, SHEETS) and the below-floor list. Nothing says
  whether a milestone's generated suite reaches each of its functions. On 2026-10-20 gen-m5 calls
  only 61 of the 131 generator-ready M5 functions (T-2108 is the fix).
- Reuse, do not re-derive (CLAUDE.md directive 3): `Catalog` (crates/rustsheet-eval/src/catalog.rs:
  `functions`, `in_scope`, `needs_other_evidence`), the per-function Excel cell counts the
  evidence section already computes (report.rs, evidence_floor.rs), `report::functions_in` for the
  calls of a formula, and `volatile.rs` for which functions are volatile (expose its name list
  if it is private).
- A generated suite is one whose suite.toml has `origin = "generated"` (crates/rustsheet-eval/src/suite.rs
  `origin`, `milestone`). Excel-backed means `from_excel` (the evidence section's rule).
  Hand-written expectations do not count.
- Expected today, for the M5 row: about 131 generator-ready functions plus INFO and SHEETS (`oracle:
  none`) and RAND/RANDBETWEEN (volatile). Condition (b) holds for about 61 of them. The Outcome
  records the real numbers.

## Acceptance criteria
- [ ] report.md has the section, with one table row per milestone: in-scope functions, meeting (a),
      meeting (b), meeting both. Below the table, each function missing (a) or (b) is listed by
      milestone with its tags. report.json carries the same data.
- [ ] Unit tests cover: a function with 20 Excel cells and one generated-suite cell (covered); one
      with 19 Excel cells (fails a); one whose only cells are in a non-generated suite (fails b); an
      `oracle: none` function and a volatile function (listed with their tags); a `scope: out`
      function (not counted).
- [ ] No second catalog parser, evidence counter or formula-call parser is added (directive 3;
      `scripts/check_duplication.py` does not rise).
- [ ] docs/architecture/eval-harness.md "Outputs" describes the section.
- [ ] The Outcome records the M5 row on the PR's tree.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deciding whether `oracle: none` and volatile functions are exempt from M5's 100% (T-2116).
- The §13 coverage matrix (edge classes, argument positions).
- Gating on the numbers.
