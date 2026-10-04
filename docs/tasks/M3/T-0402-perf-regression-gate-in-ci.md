---
id: T-0402
title: "Performance regression gate in CI"
milestone: M3
lane: perf
status: todo
priority: P1
size: S
depends_on: [T-0400]
---

## Goal
A change that makes load, full recalc, single-cell edit or memory per cell noticeably worse fails
CI, so M3/M4 start from a protected baseline.

## Context
2026-10-04 review: no gate fails on performance today; T-0400 builds the benchmarks but only
compiles them in CI. Noise on shared runners means the gate needs a stable metric (instruction
counts, e.g. iai-callgrind, or medians with a generous threshold).

## Acceptance criteria
- [ ] CI runs a fast subset (100k-cell models: load, full recalc, 1-cell edit, bytes per cell) on every PR and the full set (1M) nightly.
- [ ] A committed baseline (quality/perf.json or similar) ratchets like the other quality files: >10% regression fails, improvements are recorded with `--update`.
- [ ] docs/process/quality-gates.md documents the gate and how to update the baseline legitimately.
- [ ] Decision record for the protected-path change; `make check` green; reviewers approve.
