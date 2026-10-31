---
id: T-3360
title: "Fit lab: a mandatory deterministic hold-out split, with train and test counts side by side (decision 0188 R4.1)"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every fit-lab comparison splits its cells deterministically (about 30% held out, by case or cell hash), scores every
candidate on train and test separately, and accepts a candidate only if its test exact rate is within a stated margin
of its train rate and it beats the correctly rounded candidate on test.

## Context
- Decision 0188 R4.1 (owner's 2026-10-31 assessment): the fit lab has no hold-out, fits are scored on the cells they are
  tuned on (gated gen-m9 included), and functions fitted that way generalize badly to fresh points (T.INV 30% against
  69% on the public cases, T.INV.2T 14% against 51%, F.INV 63% against 78%).
- Code: tools/src/rustsheet_tools/fitlab, docs/architecture/fit-lab.md.

## Acceptance criteria
- [ ] The split is deterministic and documented (hash, seed, ratio); the same cell is always on the same side.
- [ ] The report shows train and test counts (bit-exact, within 4 ulp, within 15 digits) side by side for every
      candidate, the correctly rounded one included, and the accept verdict with the margin used.
- [ ] Tests cover the split's stability and the accept rule (a candidate that wins on train but not on test is refused).
- [ ] docs/architecture/fit-lab.md describes the split and the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-running existing fits (T-3365 and the generalization tasks do that).
