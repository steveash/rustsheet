---
id: T-1739
title: "Gate the displayed-text cells of gen-m10 with a text baseline"
milestone: M10
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A displayed-text cell that passes in a gated suite cannot start failing unnoticed: `make eval-check` fails on a text regression, as it does on a value regression.

## Context
Decision 0068 scores displayed text (`text` in a sidecar) apart from values and keeps it out of the
baselines and the gate, "a later decision, to be made once the suite exists". gen-m10 (T-1611) now
has 10,400 oracle text cells at 66.9%, and M10's exit criterion is the formatted-text suite at 100%,
which the gate does not measure today. Decide (decision record via `make new-decision`) and
implement a text baseline: for example a `T` line kind in eval/baseline/<suite>.txt or a sibling
file, written by `make eval-baseline` and checked by `eval-check`, with the same ratchet (fixes
recorded, regressions refused). Code: crates/rustsheet-eval (text_expect.rs, baseline handling).

## Acceptance criteria
- [ ] Decision record; `eval-check` fails on a text cell that passed in the baseline and now fails (test).
- [ ] gen-m10's failing text cells recorded with `make eval-baseline`; no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the formatter (T-1730..T-1738 and the others number-formats.md cites).
