---
id: T-1030
title: "Reconcile CLAUDE.md directive 1 and decision 0014 with non-directive-5 deviation records"
milestone: M1
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The governing documents agree on when RustSheet may deviate from Excel outside directive 5.

## Context
Raised by the Excel-fidelity reviewer on T-1025. CLAUDE.md directive 1 and decision 0014 say
directive 5 is the only exception to "Excel is always right", while docs/excel-semantics/README.md
("Deliberate deviations") allows any other deviation through its own decision record, and decisions
0033 (lone surrogates) and 0038 (WEEKNUM calc-order state) use that route. CLAUDE.md is a protected
path: the change needs its own decision record approved by reviewers.

M2 exit blocker (planner, 2026-10-13): 10 of the 196 known-failure cells assigned to M0-M2 are the decision-0033
lone-surrogate cells: probes/text-surrogate-positions A6, A11, A12, A29, A38, B10 (docs/excel-semantics/text.md
"MID and REPLACE positions inside a surrogate pair keep the lone half") and probes/concatenate-limit-spill E8-E11
(coercion.md "Truncation inside a surrogate pair keeps a lone high surrogate"). Both entries are `Status: DEVIATION
(decision 0033)`. They are not in eval/suites/probes/deviations, because decision 0036 lets that file cite only
directive-5 limits, so they count as known failures. M2 exit ("100% bit-exact, excluding accepted deviations")
cannot be reached until this task rules either way.

## Acceptance criteria
- [x] CLAUDE.md directive 1, decision 0014 and the semantics README state the same rule (either allow per-decision deviations, with what qualifies, or forbid them and reopen 0033/0038).
- [x] Decision 0036's `deviations` file scope is stated for non-directive-5 deviations.
- [x] The 10 lone-surrogate cells above have a route out of eval/baseline/probes.txt. If per-decision deviations
      are allowed, they are listed in eval/suites/probes/deviations with their pinned values and leave the baseline
      via `make eval-baseline`. If they are forbidden, decision 0033 is reopened and an engine spec task for
      lone-surrogate text is filed in M2.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Decision 0052: RustSheet deviates from Excel only where directive 5 requires it. CLAUDE.md
directive 1 and decision 0014 already said so; docs/excel-semantics/README.md no longer allows a
deviation by its own decision record. Decisions 0033 (lone surrogates) and 0038 (WEEKNUM stale
return_type) are superseded: their semantics entries are `VERIFIED` known gaps, and code comments
cite `TODO(T-1483)` / `TODO(T-1484)`. Decision 0036 is amended: `deviations` files hold only
directive-5 cells citing decision 0014. The 10 lone-surrogate cells stay F lines; their route out
of the baseline is T-1483 (engine spec, M2, P1) and its implementation tasks. Filed T-1484
(WEEKNUM calc-order state spec, M6). Eval: no change (docs and comments only).
