---
id: T-3513
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of TEXTBEFORE and TEXTAFTER"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of TEXTBEFORE and TEXTAFTER, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 369e7df4, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` on 369e7df4 lists none of them), and every function is above the evidence floor,
  but most argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error,
  blank, range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on
  those pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 369e7df4: TEXTBEFORE 0.22 (1420); TEXTAFTER 0.22 (1408). 2828 unseen pairs in all (six parameters each, so most pairs are of the optional instance_num, match_mode, match_end and if_not_found).
- Cells and passes at 369e7df4 (report.json `functions`): TEXTBEFORE 1068/1072, TEXTAFTER 959/960.
- T-3279 is the same batch for the M2 functions, and T-3492 to T-3498 for other M5, M6, M7 and M9 functions; none
  of them names the functions above. T-2876 (research, todo) probes single argument kinds of TEXTBEFORE, TEXTAFTER and TEXTSPLIT; this batch covers the pairs of the first two. T-3497 (opus) is the M7 array-shaping batch; TEXTSPLIT is left out here because its result spills, the reason T-3497 is opus. On scalar arguments TEXTBEFORE and TEXTAFTER return one value, and a forced `range` argument lifts to at most 2 x 2, which the policy's row stride leaves room for (`ROW_STRIDE` in arg_pairs.py).
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions TEXTBEFORE,TEXTAFTER`
  (`tools/src/rustsheet_tools/gen/arg_pairs.py`; T-3493 checked that the policy honours `--functions`),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`, and score.
- M7's exit counts the gen-m7 row (docs/milestones.md). The Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the
  same PR (an existing open task that covers the rule, or a new task via `make new-task` in this lane, with
  `campaign: semantics` and a `## Visible-parity stage` section for a wrong type, error or text; case file stem and
  A1 names in one bullet, decision 0159), and record any new Excel quirk in docs/excel-semantics/ citing the cells.
- Tier: size S; the generator run, the oracle fill and the coverage check are mechanical (T-3493 is the same shape at sonnet); scalar results; no fitting; fixes are out of scope.

## Acceptance criteria
- [ ] New gen-m7 cases from `--policy arg-pairs --milestone M7 --functions TEXTBEFORE,TEXTAFTER` with their excel-graph
      oracle sidecars are committed.
- [ ] After `make eval`, each of the 2 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- TEXTSPLIT and REGEXEXTRACT (spilled results; a later batch after T-3497 shows the policy handles spills).
- Single argument-kind probes (the research tasks named above) and other functions or milestones.
