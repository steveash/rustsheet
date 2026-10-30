---
id: T-3226
title: "Demote P1 tasks that are not Phase 1 work now (2026-10-30)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
P1 means Phase 1 work to pick now, so agents taking the highest-priority ready task are not steered to
work the phase plan says waits.

## Context
Follow-up to the T-0098 assessment (2026-10-28). Four open tasks were P1 although `docs/milestones.md` puts
them after Phase 1:
- T-0458 (calipers PPMT/IPMT last digits) and T-2122 (the four open gamma kernels to the bit) carry
  `defer: phase-1b`; Phase 1b is worked after Phase 1 exits and such tasks "are not picked before then"
  (decision 0114).
- T-2897 and T-2898 are M12 Python API tasks; M12 is Phase 3, after Phase 2.
The panic-safety tasks (T-2896, T-2913, T-2895) stay P1: untrusted input must never panic (CLAUDE.md
directive 5), whatever the phase.

## Acceptance criteria
- [x] T-0458 and T-2122 are P3; their `defer: phase-1b` is kept.
- [x] T-2897 and T-2898 are P2.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-ranking the research lane, and any other priority change.

## Outcome
Four front-matter priority edits; no code or eval change.
