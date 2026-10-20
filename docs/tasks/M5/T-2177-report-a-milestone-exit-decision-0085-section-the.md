---
id: T-2177
title: "Report a Milestone exit (decision 0085) section: the four M5 and M6 exit measures against their thresholds"
milestone: M5
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2175, T-2176]
---

## Goal
report.md has one "Milestone exit (decision 0085)" section with a row for M5 and a row for M6.
Each row shows the four exit measures that decision 0085 defines, next to their thresholds, and
`met` or `not met` for each, so the steward reads M5's exit status from one table. Every number
comes from a row the report already computes, so this section adds no new scoring.

## Context
- Decision 0085 (docs/decisions/0085-how-m5-s-exit-criteria-are-measured.md) defines the four
  measures, and its Consequences say M6's "same as M5" inherits them with M6 in place of M5.
  Today the steward checks four report sections by hand.
- The inputs are all fields of the report struct in crates/rustsheet-eval/src/report.rs:
  1. Catalog coverage: `catalog_coverage` row `Mk`. Met when `covered == functions` (T-2175).
  2. Generated suite: the `suites` entry named `gen-m<k>` (`SuiteSummary`). Scored is `cells`.
     The rate is `cells_passed / (cells - deviations)`. Met when scored ≥ 20,000 and the rate is
     ≥ 0.999. Also print the allowance, `floor(0.001 * (cells - deviations))`, beside the known
     failures, `cells - deviations - cells_passed`. At 22,400 cells the allowance is 22.
  3. ironcalc-calc: `function_scope.rows` row `Mk`. Met when `failing.len() - excepted.len() == 0`
     (T-2176).
  4. Probes: `probe_milestones` row `Mk`. Met when `known_failures == 0`.
- A missing input (a suite that did not run, or a row that is absent) prints `n/a`, never `met`.
  `make eval` without poi or corpus-bench must still produce the section, because none of the
  four reads them.
- Put the code in a new module, `crates/rustsheet-eval/src/milestone_exit.rs`, with
  `summarize(&Report) -> Vec<ExitRow>` and `write_markdown`, the pattern of catalog_coverage.rs.
  Write it to report.json as `milestone_exit`. Place the section before the other milestone
  sections in report.md.
- The milestones the section covers are a const list, `["M5", "M6"]`. M7 to M11 have their own
  exit criteria and need their own decision before they get a row.

## Acceptance criteria
- [ ] report.md has the section, with one row each for M5 and M6. The columns are coverage
      `covered/functions`, gen suite `scored, rate, known failures/allowance`, ironcalc-calc
      `failing excl. exceptions`, and probes `known failures`. Each measure shows `met`, `not met`
      or `n/a`, and a final `exit` column is `met` only when all four are met.
- [ ] report.json has `milestone_exit` with the same fields.
- [ ] Unit tests in milestone_exit.rs cover: all four met; each measure failing on its own (the
      gen suite at 19,999 cells, and the gen suite at a 0.998 rate); and a missing gen suite
      giving `n/a` and `exit` not met.
- [ ] On `make eval` at the PR's base, the M5 row matches the four source rows exactly, and the PR
      text quotes it.
- [ ] docs/architecture/eval-harness.md has a "Milestone exit" paragraph that cites decision 0085.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any gate or CI failure on `not met`. This is an exit measure, not a gate.
- Rows for M2 (settled by decisions 0058 and 0074) or for M7 to M11.
- Changing how any of the four source rows are computed.
- Changing the steward routine (`.claude/`, owner-only) to read the section. File that as a
  follow-up if wanted.
