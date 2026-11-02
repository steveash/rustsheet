---
id: T-3510
title: "gen-m5 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M5 text search and replace functions (FIND, SEARCH, REPLACE, SUBSTITUTE, REPT, EXACT)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m5 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of FIND, SEARCH, REPLACE, SUBSTITUTE, REPT and EXACT, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 369e7df4, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` on 369e7df4 lists none of them), and every function is above the evidence floor,
  but most argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error,
  blank, range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on
  those pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 369e7df4: FIND 0.37 (227); SEARCH 0.38 (225); REPLACE 0.32 (495); SUBSTITUTE 0.34 (479); REPT 0.31 (83); EXACT 0.33 (81). 1590 unseen pairs in all.
- Cells and passes at 369e7df4 (report.json `functions`): FIND 1188/1189, SEARCH 1133/1135, REPLACE 795/795, SUBSTITUTE 893/893, REPT 3421/3424, EXACT 654/655.
- T-3279 is the same batch for the M2 functions, and T-3492 to T-3498 for other M5, M6, M7 and M9 functions; none
  of them names the functions above. T-2863 (research, todo) probes single argument kinds of FIND, SEARCH, SUBSTITUTE and REPLACE (an error, empty text or a range); this batch covers the pairs. T-3511 is the byte-function batch (FINDB, SEARCHB, REPLACEB, ...) and T-3512 the number-to-text batch.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M5 --functions FIND,SEARCH,REPLACE,SUBSTITUTE,REPT,EXACT`
  (`tools/src/rustsheet_tools/gen/arg_pairs.py`; T-3493 checked that the policy honours `--functions`),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`, and score.
- M5's exit measures the gen-m5 row at >= 99.9% bit-exact (docs/milestones.md, decision 0085). The Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the
  same PR (an existing open task that covers the rule, or a new task via `make new-task` in this lane, with
  `campaign: semantics` and a `## Visible-parity stage` section for a wrong type, error or text; case file stem and
  A1 names in one bullet, decision 0159), and record any new Excel quirk in docs/excel-semantics/ citing the cells.
- Tier: size S; the generator run, the oracle fill and the coverage check are mechanical (T-3493 is the same shape at sonnet); scalar text functions whose results never spill on scalar arguments; no fitting; fixes are out of scope.

## Acceptance criteria
- [ ] New gen-m5 cases from `--policy arg-pairs --milestone M5 --functions FIND,SEARCH,REPLACE,SUBSTITUTE,REPT,EXACT` with their excel-graph
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
- Single argument-kind probes (the research tasks named above) and other functions or milestones.

## Landing notes
Session stopped early at the owner's request; no PR opened. Work is on branch `agent/functions-text/T-3510-m5-text-arg-pairs`.
- Done: ran `make eval`, then `rustsheet-gen --policy arg-pairs --milestone M5 --functions FIND,SEARCH,REPLACE,SUBSTITUTE,REPT,EXACT`
  (18 new gen-m5 cases) and `rustsheet-oracle fill -b excel-graph --suite gen-m5` (18 sidecars, 0 failures). Both are committed.
- Result of `make eval-check` on the new cases: 0 regressions in the baseline, but 23 new-coverage failures
  (gen-m5 known failures 1 -> 24) and four visible-parity REGRESSION lines: 5136e04c4ce978c3 (8 cells),
  5614fae2486f6dd5 (7), edae081ad96b1d56 (6), f01da2cf9f4f2105 (2); the lock allows 0 for each.
- Not done: I did not check the `pairwise` share in report.json `argument_coverage` against the 0.9 target (18 cases
  is probably too few; the policy may need more runs or a bigger count), did not run `make eval-baseline`, did not own the
  failing cells (existing open task, or `make new-task` in this lane with `campaign: semantics` and a
  `## Visible-parity stage` section naming each case stem and A1 names, decision 0159), did not record quirks in
  docs/excel-semantics/, and did not run `make check`, the reviewers or open the PR.
- Next step: `make eval-show CASE=<id>` on the four visible-parity cases and the 23 failing cells, group them by rule,
  find or file the owning tasks, then `make eval-baseline` and `make unowned-off ALL=1`.
- Do not hand-edit eval/baseline/ or the visible-parity lock.
