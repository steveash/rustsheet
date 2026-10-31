---
id: T-3362
title: "Reviewer and skill text for the anti-overfitting guards: hold-out counts, Gate A envelope, fresh sweep, reproducibility floor (decision 0188 R4)"
milestone: M0
lane: quality
status: in-progress
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
- [ ] reviewer-excel-fidelity's checklist requires, for any fit: train and test counts from the fit lab's hold-out
      (R4.1), the fresh-sweep result (R4.2), the Gate A envelope result (R2), the `FITTED` tag with its range (R4.3), and
      that no step fits below the reproducibility floor (decision 0112; R4.4). A missing item is a REQUEST CHANGES.
- [ ] reviewer-excel-fidelity's deviation check (and docs/process/quality-gates.md) accept a difference from Excel
      only for directive 5 (`DEVIATION`, decision 0014) or a `MATH-OVER-EXCEL` region listed in decision 0188 R3.
- [ ] The work-loop and steward skills state the same guards where they describe fit work; the gap-triager adds them
      to the acceptance criteria of the fit tasks it files.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fit lab's split itself (T-3360) and the tag ratchet (T-3361).
