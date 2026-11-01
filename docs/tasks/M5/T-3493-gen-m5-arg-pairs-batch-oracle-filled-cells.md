---
id: T-3493
title: "gen-m5 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M5 rounding and integer math functions (TRUNC, MOD, QUOTIENT, MROUND, CEILING.MATH, FLOOR.MATH, ECMA.CEILING, POWER, LOG, ATAN2, COMBIN, COMBINA)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m5 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of TRUNC, MOD, QUOTIENT, MROUND, CEILING.MATH, FLOOR.MATH, ECMA.CEILING, POWER, LOG, ATAN2, COMBIN and COMBINA, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: TRUNC 0.40 (73); MOD 0.50 (61); QUOTIENT 0.40 (73); MROUND 0.54 (56); CEILING.MATH 0.53 (170); FLOOR.MATH 0.56 (161); ECMA.CEILING 0.41 (71); POWER 0.52 (58); LOG 0.40 (73); ATAN2 0.44 (68); COMBIN 0.27 (88); COMBINA 0.22 (94). 1046 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). T-2862 and T-2871 (research) probe single argument kinds of M5 math value parameters; this batch covers the pairs. T-3494 is the other half of the M5 math functions.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M5 --functions TRUNC,MOD,QUOTIENT,MROUND,CEILING.MATH,FLOOR.MATH,ECMA.CEILING,POWER,LOG,ATAN2,COMBIN,COMBINA`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`, and score.
- M5's exit measures the gen-m5 row at ≥ 99.9% bit-exact (docs/milestones.md, decision 0085; met at 32c4a203),
  and the Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- Tier: size S, the run and the coverage check are mechanical (T-3279 is the same shape and tier), no fitting, fixes out of scope.

## Acceptance criteria
- [ ] New gen-m5 cases from `--policy arg-pairs --functions TRUNC,MOD,QUOTIENT,MROUND,CEILING.MATH,FLOOR.MATH,ECMA.CEILING,POWER,LOG,ATAN2,COMBIN,COMBINA` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 12 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M5` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.

## Outcome
`rustsheet-gen --policy arg-pairs` for the 12 functions wrote 14 gen-m5 cases; `rustsheet-oracle fill -b excel-graph`
filled all 14 sidecars (0 failures). After `make eval`, `argument_coverage` pairwise is 1.0 (0 unseen pairs) for
TRUNC, MOD, QUOTIENT, MROUND, CEILING.MATH, FLOOR.MATH, ECMA.CEILING, POWER, LOG, ATAN2, COMBIN and COMBINA.
Every new cell passes: no new `F` lines in eval/baseline/gen-m5.txt (only 14 `C` case lines), so no new owners
are needed; `make exit-owners MILESTONE=M5` reports 0 unowned cells. gen-m5: 47984 cells, 47983 bit-exact
(the remaining cell is a pre-existing deviation, not from this batch). No new Excel quirk found.
