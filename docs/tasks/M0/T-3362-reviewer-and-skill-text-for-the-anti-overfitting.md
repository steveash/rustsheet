---
id: T-3362
title: "Reviewer and skill text for the anti-overfitting guards: hold-out counts, Gate A envelope, fresh sweep, reproducibility floor (decision 0188 R4)"
milestone: M0
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The excel-fidelity reviewer refuses a numeric fit without its hold-out counts and Gate A envelope result, and the
work-loop and steward skills tell workers the R4 guards: the hold-out split, the fresh log-uniform sweep before merge,
the `FITTED` tag and its range, and no fitting below the oracle's reproducibility floor.

## Context
- Decision 0188 R4.2, R4.4 and R4.5 (owner's 2026-10-31 assessment: fits scored on their own tuning cells generalize
  badly to fresh points).
- Files: .claude/agents/reviewer-excel-fidelity.md (its check list), .claude/skills/work-loop/SKILL.md (the numeric-fit
  steps), .claude/skills/steward/SKILL.md (triage of fit tasks). Also .claude/agents/gap-triager.md where it files
  fit tasks, so new fit tasks carry the guards in their acceptance criteria. These are protected paths: the PR cites
  decision 0188.

## Acceptance criteria
- [x] reviewer-excel-fidelity's checklist requires, for any fit: train and test counts from the fit lab's hold-out
      (R4.1), the fresh-sweep result (R4.2), the Gate A envelope result (R2), the `FITTED` tag with its range (R4.3), and
      that no step fits below the reproducibility floor (decision 0112; R4.4). A missing item is a REQUEST CHANGES.
- [x] reviewer-excel-fidelity's deviation check (and docs/process/quality-gates.md) accept a difference from Excel
      only for directive 5 (`DEVIATION`, decision 0014) or a `MATH-OVER-EXCEL` region listed in decision 0188 R3.
- [x] The work-loop and steward skills state the same guards where they describe fit work; the gap-triager adds them
      to the acceptance criteria of the fit tasks it files.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fit lab's split itself (T-3360) and the tag ratchet (T-3361).

## Outcome
- reviewer-excel-fidelity check 6: a difference from Excel only for directive 5 or a `MATH-OVER-EXCEL` case listed in
  decision 0188 R3 or a later owner-ruled record, with evidence points and bounds committed; R3's delegated bound
  fixing (tiny-p, T-3358) allowed; other new or widened regions, R3 against an error, or a math path past directive 5's
  caps are BLOCKING.
- New check 7 "Numeric fits" holds the one full list: three tiers (a fit; a verbatim cited constant, not a fit; a
  structural model, owing hold-out and fresh sweep but no tag), then the guards (hold-out with the margin fixed before
  scoring and the fit lab's ranking, fresh sweep with SPEC-0029's batch rules, Gate A envelope per R2, `FITTED` tag and
  range, reproducibility floor, no keep-matched branch on decisions 0056 and 0082) and the interim wording until
  T-3356, T-3357, T-3358, T-3360 and T-3361 land.
- work-loop step 4, the steward's "Fit guards" and the gap-triager point to check 7 with a short summary; the steward
  reads decision 0056 with 0188 R4.1 (an attempt counts only if it raises the hold-out bit-exact count), as decision
  0188's Consequences now say. docs/process/quality-gates.md states the accepted differences.
- No code, eval or baseline change (text only; typos, hygiene, task, decision and governance checks pass locally).
