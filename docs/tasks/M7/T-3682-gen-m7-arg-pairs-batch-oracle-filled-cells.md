---
id: T-3682
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of SEQUENCE"
milestone: M7
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of SEQUENCE, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main c0e330c2f; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). No SEQUENCE cell is visibly off and `make unowned-off ALL=1` lists none
  for it. It is above the evidence floor. Yet most argument-class pairs of its four parameters (number, zero,
  negative, fraction, numeric_text, text, empty_text, boolean, error, blank, range;
  `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all. Those pairs are unverified, however many
  cells pass.
- At c0e330c2f, SEQUENCE has a single-kind share of 0.84 and a pairwise share of 0.22, with 568 unseen pairs. The first
  ones are rows/columns = (number, numeric_text), (number, empty_text) and (number, error). SEQUENCE is the M7 math
  function with the lowest pairwise share. RANDARRAY (0.05) is left out because it is volatile, so the harness skips
  its cells.
- Related tasks:
  - T-3497 (done) ran this batch for the M7 array-shaping functions.
  - T-3279 is the M2 batch and T-3494 the M5 math batch.
  - T-2874 (research, todo) probes one argument kind per SEQUENCE parameter (empty text, an error). This batch covers
    the pairs, so the two do not overlap.
  - T-2608 (research, todo) owns the sizes past the grid.
- Dry run on c0e330c2f, into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions SEQUENCE` wrote 6 cases and
  1136 formulas, such as `SEQUENCE(Data!A1:B2,"2.5")`, `SEQUENCE(Data!C3:D4,Data!G9)` and `SEQUENCE(Data!A1:B2,#VALUE!)`.
- Why opus: the run needs generator judgment before its cells can be trusted.
  - A rows or columns draw from the number pool (7, 123456789, a range lifted into an array of sizes) can give a
    result taller than T-3497's 61-row `SPILL_ROW_STRIDE`. It can also run into the grid edge or the engine's size
    limits (directive 5, T-2608). A `#SPILL!` from the layout or a DEVIATION must not be what gets scored.
  - A multi-cell range in rows or columns lifts SEQUENCE into a nested array, which is `#CALC!` in Excel.
- Steps:
  1. Run `make eval` so the policy reads the current unseen pairs.
  2. Generate with `--spill` (check the footprint stride fits the results).
  3. Fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`, and score.
- M7's exit needs gen-m7 at 99.9% or more, with 0 spill-extent failures, at visible parity (docs/milestones.md,
  decision 0140). The Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters. Own each one in
  the same PR:
  - Use an existing open task that covers the rule, or file a new task with `make new-task` in this lane.
  - A wrong type, error or text gets `campaign: semantics` and a `## Visible-parity stage` section.
  - Name the case file stem and the A1 cells in one bullet (decision 0159).
  - Record any new Excel quirk in docs/excel-semantics/ (arrays.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m7 cases from `--policy arg-pairs --functions SEQUENCE` are committed with their excel-graph oracle
      sidecars and 0 fill failures.
- [ ] After `make eval`, SEQUENCE has a `pairwise` share of at least 0.9 in report.json `argument_coverage`. Otherwise
      the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load, a size past the grid).
- [ ] No scored cell of the batch is a `#SPILL!` caused by the case layout. Spill-extent failures in the M7 exit row
      stay at 0.
- [ ] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions). Each is owned
      by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no new unowned cell from
      the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds. The owning tasks make them. The exception is a one-line coercion fix in
  `functions/math_and_trigonometry/sequence.rs` that the PR also tests.
- RANDARRAY (volatile), the single-kind probes of T-2874, the size edges of T-2608, and other functions or milestones.
