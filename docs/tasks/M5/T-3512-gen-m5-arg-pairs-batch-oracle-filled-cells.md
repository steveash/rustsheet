---
id: T-3512
title: "gen-m5 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M5 number-to-text and join functions (FIXED, DOLLAR, NUMBERVALUE, VALUETOTEXT, ARRAYTOTEXT, TEXTJOIN)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m5 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of FIXED, DOLLAR, NUMBERVALUE, VALUETOTEXT, ARRAYTOTEXT and TEXTJOIN, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 369e7df4, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` on 369e7df4 lists none of them), and every function is above the evidence floor,
  but most argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error,
  blank, range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on
  those pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 369e7df4: FIXED 0.25 (272); DOLLAR 0.26 (89); NUMBERVALUE 0.25 (271); VALUETOTEXT 0.32 (82); ARRAYTOTEXT 0.26 (89); TEXTJOIN 0.53 (172). 975 unseen pairs in all.
- Cells and passes at 369e7df4 (report.json `functions`): FIXED 739/739, DOLLAR 685/686, NUMBERVALUE 725/728, VALUETOTEXT 654/654, ARRAYTOTEXT 893/893, TEXTJOIN 4164/4182.
- T-3279 is the same batch for the M2 functions, and T-3492 to T-3498 for other M5, M6, M7 and M9 functions; none
  of them names the functions above. T-2863 (research, todo) probes single argument kinds of FIXED, DOLLAR and TEXTJOIN; this batch covers the pairs. TEXT (0.57) is left out on purpose: its format-code argument belongs to the M10 number-format tasks, and a pair batch there needs format-code triage.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M5 --functions FIXED,DOLLAR,NUMBERVALUE,VALUETOTEXT,ARRAYTOTEXT,TEXTJOIN`
  (`tools/src/rustsheet_tools/gen/arg_pairs.py`; T-3493 checked that the policy honours `--functions`),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`, and score.
- M5's exit measures the gen-m5 row at >= 99.9% bit-exact (docs/milestones.md, decision 0085). The Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the
  same PR (an existing open task that covers the rule, or a new task via `make new-task` in this lane, with
  `campaign: semantics` and a `## Visible-parity stage` section for a wrong type, error or text; case file stem and
  A1 names in one bullet, decision 0159), and record any new Excel quirk in docs/excel-semantics/ citing the cells.
- Tier: size S; the generator run, the oracle fill and the coverage check are mechanical (T-3494 is the same shape at sonnet, with array-mode parameters too); no fitting; fixes are out of scope.

## Acceptance criteria
- [ ] New gen-m5 cases from `--policy arg-pairs --milestone M5 --functions FIXED,DOLLAR,NUMBERVALUE,VALUETOTEXT,ARRAYTOTEXT,TEXTJOIN` with their excel-graph
      oracle sidecars are committed.
- [ ] After `make eval`, each of the 6 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M5` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- TEXT and its format-code pairs (M10 number-format work).
- Single argument-kind probes (the research tasks named above) and other functions or milestones.
