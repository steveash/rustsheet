---
id: T-3346
title: "Record the owner's Phase 1 two-gate exit rulings"
milestone: M0
lane: quality
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
The repository records the owner's 2026-10-31 rulings (decision 0188): Phase 1 exits on an
accuracy gate (Gate A) and user-representative fifteen-digit parity (Gate B), Excel is followed
except where it is grossly wrong against the math, numeric fits are guarded against overfitting,
and the tasks that implement all of it are filed.

## Context
Owner's 2026-10-31 assessment of a fresh eval at `main` 921ed6ad (decision 0188 gives the numbers).

## Acceptance criteria
- [ ] Decision 0188 records rulings R1-R7.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine, harness or eval change: the filed tasks do that.
