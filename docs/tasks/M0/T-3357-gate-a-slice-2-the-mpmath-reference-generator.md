---
id: T-3357
title: "Gate A slice 2: the mpmath reference generator, the stratified sampler and the oracle fill in tools/"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3355]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
A tool in tools/ writes Gate A reference tables (T-3355's format): it draws stratified points per function (about 40%
typical, 30% tails and domain edges, 20% extreme-but-valid, 10% log-uniform; decision 0188 R2), computes each point's
50-digit mpmath value, and has the oracle fill Excel's value.

## Context
- Decision 0188 R2; design in the Gate A spec (T-3355).
- Reuse the oracle client (`rustsheet-oracle`), the spec→xlsx builder and `rustsheet-eval snapshot`; Python is ruff
  and mypy strict; never re-implement xlsx value parsing (CLAUDE.md conventions).

## Acceptance criteria
- [ ] The generator is deterministic (same seed, same points), and its mpmath definitions for at least three functions
      (one distribution, one inverse, one financial solver) are unit-tested against published values.
- [ ] The fill writes Excel's values into a table and leaves unfilled points pending, as the spec says.
- [ ] `make check` green (ruff, mypy, tests); reviewers approve.

## Out of scope
- The make check gate (T-3356); generating every function's table (T-3358).
