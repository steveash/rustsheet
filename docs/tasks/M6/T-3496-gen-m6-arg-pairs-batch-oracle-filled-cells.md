---
id: T-3496
title: "gen-m6 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M6 lookup functions (ADDRESS, OFFSET, INDIRECT, XMATCH, XLOOKUP, CHOOSE, CHOOSEROWS, CHOOSECOLS, TRIMRANGE, HYPERLINK)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m6 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of ADDRESS, OFFSET, INDIRECT, XMATCH, XLOOKUP, CHOOSE, CHOOSEROWS, CHOOSECOLS, TRIMRANGE and HYPERLINK, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: ADDRESS 0.15 (1031); OFFSET 0.32 (520); INDIRECT 0.29 (86); XMATCH 0.37 (251); XLOOKUP 0.46 (439); CHOOSE 0.54 (56); CHOOSEROWS 0.44 (68); CHOOSECOLS 0.47 (64); TRIMRANGE 0.26 (106); HYPERLINK 0.21 (96). 2717 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). GETPIVOTDATA (0.05) is left out: its pairs need a pivot table the generator does not build. T-3497 is the M7 lookup batch.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M6 --functions ADDRESS,OFFSET,INDIRECT,XMATCH,XLOOKUP,CHOOSE,CHOOSEROWS,CHOOSECOLS,TRIMRANGE,HYPERLINK`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6`, and score.
- M6's exit measures gen-m6 the way M5's measures gen-m5 (≥ 99.9% bit-exact, docs/milestones.md, decision 0085),
  and the Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- OFFSET and INDIRECT results that spill or reference outside the Data sheet are fine as long as the case scores them; drop a draw only if Excel refuses to load the workbook.
- Tier: size S, the run and the coverage check are mechanical (T-3279 is the same shape and tier), no fitting, fixes out of scope.

## Acceptance criteria
- [ ] New gen-m6 cases from `--policy arg-pairs --functions ADDRESS,OFFSET,INDIRECT,XMATCH,XLOOKUP,CHOOSE,CHOOSEROWS,CHOOSECOLS,TRIMRANGE,HYPERLINK` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 10 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m6.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M6` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.

## Outcome
32 arg-pairs cases added to gen-m6 with excel-graph sidecars; pairwise share 1.0 (0 unseen pairs) for all 10 functions. gen-m6 30652/30674 bit-exact:
22 new known failures, owned by T-3541 (CHOOSEROWS/CHOOSECOLS, 18 cells) and T-3542 (XMATCH, 4 cells). The visible-parity lock was recorded by `make eval-baseline` as new coverage (decision 0088).
