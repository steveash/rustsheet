---
id: T-3429
title: "Progress dashboard shows Gate B, Gate A and the Phase 1b evidence counts (decision 0188)"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-3359]
---

## Goal
docs/progress.md shows Gate B (met / not met / n/a, functions at rate, with the every-suite count as information),
Gate A (from `eval/runs/accuracy.json`; `n/a` when absent) and the Phase 1b evidence counts once T-3427 lands.

## Context
- Split out of T-3359 (criterion 6). Gate B is in the eval report JSON at `v15_checkpoint.gate_b` (`measured`,
  `missing`, `in_scope`, `at_rate`, `met`, `functions`) and in `v15_checkpoint.phase1_line`.
- Generator: tools/src/rustsheet_tools/progress_report/ (SPEC-0022, the `progress-report` skill); `model.py` reads
  `at_rate` and `phase1_met` today.

## Acceptance criteria
- [ ] The dashboard's Phase 1 block shows Gate B and, beside it, the every-suite `at_rate` as information; a report
      without `gate_b` (older runs) shows `n/a`.
- [ ] A Gate A line reads `eval/runs/accuracy.json` when present (`n/a` otherwise).
- [ ] The Phase 1b evidence counts appear once T-3427's report fields exist (`n/a` before).
- [ ] tools/tests cover each with a fixture report.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what the measures count (T-3359, T-3427).
