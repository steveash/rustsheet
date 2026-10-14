---
id: T-1523
title: "Fit lab: score candidate kernel algorithms against the oracle sweeps, bit-exact and within 4 ulp"
milestone: M9
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A reusable fit lab: give it a sweep (oracle-filled probe case) and candidate implementations of a kernel, and it reports per candidate how many cells are bit-exact and within 4 ulp, with the worst cells, so a fit is a ranked comparison instead of guesswork.

## Context
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The oracle sweeps for every major kernel exist (T-1040 to T-1046, T-1004 to T-1009, `eval/suites/probes-numeric/cases/m9-*`). `docs/research/t1187-erfc-candidates.py` is a one-off version of this for erfc. The fit-without-sweep rule (decision 0026) and the time-box (decision 0056) both depend on it.

## Acceptance criteria
- [ ] `tools/src/rustsheet_tools/fitlab/` (uv project, ruff and mypy strict): reads sweep inputs and Excel values through the existing readers (`tools/src/rustsheet_tools/snapshot.py`, the oracle package's sidecar reading; never re-parsing xlsx), runs candidate functions, prints a ranked table and writes JSON.
- [ ] Exact arithmetic helpers: correctly rounded results from `fractions`, rounding to a p-bit significand (53 and 64), so candidates can model x87 double rounding.
- [ ] One worked example: the engine's current kernel and at least two candidates for one kernel, run on its sweep.
- [ ] It replaces `docs/research/t1187-erfc-candidates.py`: those candidates are ported into the lab and the script is removed.
- [ ] Documented in `docs/architecture/` (how to add a candidate).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting a winning candidate into the engine (the kernel's own fit task does that).
