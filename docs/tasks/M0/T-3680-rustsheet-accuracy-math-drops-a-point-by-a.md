---
id: T-3680
title: "rustsheet-accuracy math drops a point by a 10 s wall clock, so sidecars depend on the machine: use a deterministic budget"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math` writes the same sidecars on any machine for the same inputs, as `write_suite` claims.

## Context
- tools/src/rustsheet_tools/accuracy/tables.py `TIME_LIMIT = 10.0` drops a point whose `compute()` takes more than 10 s
  of wall-clock time.
- On 2026-11-01, regenerating probes-numeric for T-3529's merge dropped
  probes-numeric/t3350-r4-chisq-inv-sweep Calc!A237 (`=CHISQ.INV.RT(0.999999999962187,9123239)`). A faster run had
  written it (9095462.568446318...). The commit restored the earlier output by hand from git, since the case is
  unchanged. A second run under load the same day dropped t1531-t-tiny-p Calc!A106 (`=T.INV.2T(1E-300,100000)`)
  the same way.
- A machine-dependent drop silently unchecks a cell against the math, and it makes sidecars differ between runs.
  Gate A table batches use the same limit.

## Acceptance criteria
- [ ] Points are dropped by a deterministic budget (iterations, terms or precision steps), not wall-clock time; or the
      limit is generous enough that no committed point is near it, with a test that pins A237.
- [ ] Regenerating probes-numeric twice on different load changes no byte.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the incomplete-gamma series.
