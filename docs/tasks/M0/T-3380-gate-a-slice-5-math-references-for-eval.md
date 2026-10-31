---
id: T-3380
title: "Gate A slice 5: math references for eval cells (.math.json sidecars read by the runner, the mpmath fill for probes-numeric sweeps)"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3356, T-3357]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Eval cells can carry a 50-digit math value, so decision 0188 R1(c) counts a visibly-off numeric probe cell within
ε = 1e-9 of the math as inside Gate A's envelope, and the R5-deferred probes-numeric sweep cells stop counting against
the milestone exits (SPEC-0029, "Math references for eval cells").

## Context
- SPEC-0029 (T-3355): the `rustsheet-math/v1` sidecar `<case>.math.json` next to the oracle sidecar, each cell with its
  formula text `f`; `envelope` in `crates/rustsheet-eval/src/accuracy.rs` (T-3356); the definitions of T-3357.
- Reuse the fit lab's sweep reader (`tools/src/rustsheet_tools/fitlab/sweep.py`) to parse arguments, and
  `rustsheet-eval cases` to list cases (tools never parse xlsx values).
- T-3359 reads the math value through `envelope` for R1(c); without one a visibly-off numeric cell counts as outside.

## Acceptance criteria
- [ ] `rustsheet-accuracy math --suite S` writes math sidecars for every checked one-call cell with a definition and
      parseable arguments, and skips the rest; it is deterministic.
- [ ] The runner reads a math sidecar when present, puts the value on `CellResult` (absent from JSON when none) and
      drops a value whose formula text differs; unit tests as in SPEC-0029's test plan.
- [ ] Math sidecars for the probes-numeric cases of the functions T-3358 defines are committed; the Outcome reports
      how many visibly-off numeric cells are inside the envelope and how many have no math value.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The report rows that use it (T-3359); definitions beyond T-3358's batch (their own slices).
