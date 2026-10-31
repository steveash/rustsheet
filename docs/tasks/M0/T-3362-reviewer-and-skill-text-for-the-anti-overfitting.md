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
- reviewer-excel-fidelity: check 6 accepts a difference from Excel only for directive 5 or a `MATH-OVER-EXCEL`
  region decision 0188 R3 lists (a new or widened region is BLOCKING); new check 7 "Numeric fits" makes a missing or
  failed item a REQUEST_CHANGES: hold-out counts beating the correctly rounded candidate (R4.1), the fresh log-uniform
  sweep (R4.2), the Gate A envelope result (R2), the `FITTED` tag and its range (R4.3), the reproducibility floor
  (R4.4, decision 0112), and no branch kept only to keep old cells matched.
- work-loop step 4 "Numeric fits", the steward's "Fit guards" (plus: a time-box attempt counts only if it raises the
  hold-out count), the gap-triager's fit-task acceptance criteria, and docs/process/quality-gates.md's accepted
  differences state the same guards. Decision 0188's Consequences point to them.
- Interim wording until the tools land: a manual ~30% hold-out split (T-3360), a new probes-numeric case as the fresh
  sweep (T-3357, T-3358), mpmath values attached in the task as the envelope (T-3356), the tag form of T-3361.
- No code, eval or baseline change; `make check` is left to CI (text only; typos, hygiene, task and decision checks
  pass locally).
