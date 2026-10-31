---
id: T-3464
title: "gen-m2 typed-input batch: oracle-filled typed entries from --policy typed-input, every failing cell owned"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3462, T-3463]
---

## Goal
gen-m2 gains an oracle-filled batch of typed entries from `rustsheet-gen --policy typed-input` (T-3462 numbers,
T-3463 dates and clocks), and every Excel cell of it the engine fails is owned by an open task.

## Context
- The policy and its strata are documented in docs/architecture/synthetic-generator.md (T-3462, T-3463). This task
  runs it, fills it and scores it, as T-3279 does for `--policy arg-pairs`.
- Steps: `make eval`; `cd tools && uv run rustsheet-gen --policy typed-input --seed 0` (the suite is gen-m2);
  `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite gen-m2`; `make eval`;
  `make eval-show CASE=gen-m2/<case>` for each failing case.
- M2's exit needs gen-m2 with no visibly-off cell (docs/milestones.md, decision 0135), so every failing cell
  holds M2 open until it is fixed. Own each one in this PR: an open task that covers the rule (add the cells to
  it), or a new engine task via `make new-task` (milestone M2, lane engine), with the case file stem and the A1
  names in one bullet (decision 0159). `make exit-owners MILESTONE=M2` must report 0.
- An Excel answer that no semantics entry records (a shape the grammar read or refused that the docs do not
  cover) goes into docs/excel-semantics/ (coercion.md "Typed currency and percent formats", dates.md, numbers.md)
  in this PR, with the case and cells as evidence (CLAUDE.md directive 6).
- Entries Excel for the web refused (PATCH 400) are blank in Excel: they are answers, not fill failures. List
  how many there were in the PR.

## Acceptance criteria
- [ ] The typed-input cases with their excel-graph oracle sidecars are committed under eval/suites/gen-m2/.
- [ ] Failing cells are known failures in eval/baseline/gen-m2.txt (`make eval-baseline`, 0 regressions), and
      `make exit-owners MILESTONE=M2` reports 0.
- [ ] The PR lists the visibly-off count per stratum and every task it filed or extended.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells the batch finds (the owning tasks).
- Changing the policy (follow-up to T-3462 or T-3463 if it draws something wrong).
