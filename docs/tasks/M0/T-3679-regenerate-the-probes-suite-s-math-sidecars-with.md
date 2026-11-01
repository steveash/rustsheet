---
id: T-3679
title: "Regenerate the probes suite's math sidecars with the beta, gamma and discrete definitions (after #2698 and #2668)"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3587, T-3529]
---

## Goal
Every probes-suite case with a one-call cell of a Gate A function has a math sidecar that covers BETA.DIST, BETA.INV,
GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, BINOM.DIST, NEGBINOM.DIST, POISSON.DIST and the cash-flow functions. Then decision
0188 R1(c), and decision 0227's rate, read those cells against the math too.

## Context
- T-3587 (#2698) wrote the probes sidecars before T-3529's beta and gamma definitions were on main.
- T-3529 (#2668) regenerated only probes-numeric. Regenerating probes in the same PR would have conflicted add/add with
  #2698.
- Run `cd tools && uv run rustsheet-accuracy math --suite probes` on main once both have merged, and commit the
  result.

## Acceptance criteria
- [ ] The probes sidecars are regenerated on main; running the tool again changes no byte (T-3680 permitting).
- [ ] `make eval` quotes the R1(c) line before and after; `make eval-baseline` records nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New Gate A definitions.
