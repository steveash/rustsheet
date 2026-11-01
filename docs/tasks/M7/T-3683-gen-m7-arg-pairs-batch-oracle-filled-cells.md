---
id: T-3683
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of LET, MAKEARRAY, REDUCE and SCAN"
milestone: M7
lane: functions-logic
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of LET, MAKEARRAY, REDUCE and SCAN, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main c0e330c2f; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). None of these functions has a visibly-off cell or an unowned failing
  cell (`make unowned-off ALL=1`), and all are above the evidence floor. Yet most of their argument-class pairs
  (`crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell, so their behaviour on those pairs is
  unverified.
- Pairwise share and unseen pairs at c0e330c2f:

  | function | pairwise share | unseen pairs | first unseen pairs |
  |---|---|---|---|
  | LET | 0.50 | 60 | name_value/calculation = (number, empty_text), (number, error) |
  | MAKEARRAY | 0.21 | 96 | rows/columns = (number, empty_text), (number, error), (zero, zero) |
  | REDUCE | 0.43 | 69 | initial_value/array = (number, zero), (number, negative), (number, empty_text) |
  | SCAN | 0.38 | 75 | — |

  That is 300 unseen pairs in all. BYROW, BYCOL, MAP, ISOMITTED and LAMBDA have no pairwise gap.
- Related tasks:
  - T-3497 (done) ran this batch for the M7 array-shaping functions. It left out GROUPBY and PIVOTBY, whose pairs
    mostly involve LAMBDA arguments (lookup lane).
  - T-3492 is the M5 logic batch (SWITCH, IFS, IFNA).
  - T-2878 (research, todo) probes one argument kind per LET, REDUCE and SCAN parameter. T-2874 does the same for
    MAKEARRAY's rows and columns. This batch covers the pairs, so they do not overlap.
- Dry run on c0e330c2f, into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions LET,MAKEARRAY,REDUCE,SCAN` wrote
  4 cases and 600 formulas. Examples:
  - `SCAN(2,-1,_xleta.PRODUCT)` (an eta-reduced function)
  - `SCAN(-1,-1,LAMBDA(z,[v],ISOMITTED(v)))` (an optional LAMBDA parameter)
  - `SCAN(0,Data!H1,LAMBDA(w,y,Data!D$11))` (a LAMBDA body that is a reference)
- Why opus: the drawn `function` arguments mix eta-reduced names, optional `[x]` parameters and LAMBDAs whose body
  ignores its parameters or returns a reference. MAKEARRAY spills, and a sized draw can outgrow T-3497's 61-row
  stride. Telling whether a failure is a generator artefact (an unloadable formula, a layout `#SPILL!`, a size limit
  under directive 5) or a real Excel rule takes judgment before cells are owned.
- Steps:
  1. Run `make eval` so the policy reads the current unseen pairs.
  2. Generate with `--spill`.
  3. Fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`, and score.
- M7's exit needs gen-m7 at 99.9% or more, with 0 spill-extent failures, at visible parity (docs/milestones.md,
  decision 0140). The Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR:
  - Use an existing open task that covers the rule, or file a new task with `make new-task`. The lane is
    functions-logic for `functions/logical/{let,map_reduce,scan}.rs`. A fix in LAMBDA binding or evaluation goes to
    engine.
  - A wrong type, error or text gets `campaign: semantics` and a `## Visible-parity stage` section.
  - Name the case file stem and the A1 cells in one bullet (decision 0159).
  - Record any new Excel quirk in docs/excel-semantics/ (arrays.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m7 cases from `--policy arg-pairs --functions LET,MAKEARRAY,REDUCE,SCAN` are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 4 functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`. Otherwise the PR lists the pairs left unseen and why.
- [ ] No scored cell of the batch is a `#SPILL!` caused by the case layout. Spill-extent failures in the M7 exit row
      stay at 0.
- [ ] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions). Each is owned
      by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no new unowned cell from
      the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds. The owning tasks make them. The exception is a one-line coercion fix in
  the function's own module that the PR also tests.
- GROUPBY/PIVOTBY pairs, the single-kind probes of T-2878 and T-2874, and other functions or milestones.
