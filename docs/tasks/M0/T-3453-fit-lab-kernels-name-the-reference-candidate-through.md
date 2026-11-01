---
id: T-3453
title: "Fit lab: kernels name the reference candidate through holdout.REFERENCE; pin within_fifteen_digits against closeness.rs vectors"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3360]
---

## Goal
Renaming a kernel's reference candidate cannot quietly switch off the fit lab's accept rule, and the
Python 15-digit count cannot drift from the harness's.

## Context
- T-3360 (PR #2586) added `fitlab/holdout.py`. Its accept rule compares every candidate with the candidate named
  `REFERENCE = "correctly rounded"`. The kernels still spell that name as a literal (kernels/gamma.py, kernels/
  power.py, kernels/norm_inv.py, kernels/annuity.py, kernels/beta.py and others; `rg '"correctly rounded"'
  tools/src`). A fit whose reference is renamed accepts nothing: that fails safe, but silently. Review finding,
  quality and architecture MINOR on #2586.
- T-3360 also added `score.within_fifteen_digits`, a port of `crates/rustsheet-eval/src/closeness.rs`
  `within_fifteen_digits`. Only Python unit tests pin it.

## Acceptance criteria
- [ ] Every kernel that builds the correctly rounded candidate names it through `holdout.REFERENCE`, and no
      `"correctly rounded"` literal remains as a candidate name under `tools/src/rustsheet_tools/fitlab/kernels/`.
- [ ] A shared set of (expected, actual, within) vectors is checked by both a Rust unit test of
      `closeness::within_fifteen_digits` and a Python test of `score.within_fifteen_digits`, so the two cannot
      drift. Use one fixture file both read, or the same literal table in both, with a comment naming the other.
- [ ] Optional: `Fit` refuses a margin outside [0, 1] or a non-finite one (security MINOR on #2586).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the split, the margin or the accept rule.
