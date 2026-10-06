---
id: T-0233
title: "Probe CONCATENATE 32767-character limit and array spill values"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for CONCATENATE's 32767-character limit and for the element values of a spilled CONCATENATE (arrays.md entries marked UNVERIFIED), so the implementation can be confirmed or corrected.

## Context
Found by the T-0226 fidelity review. Use the `excel-probe` skill. Code: `functions/text/concatenate.rs`.

## Acceptance criteria
- [ ] Probe cases (with room to spill) for 32767 vs 32768 chars, scalar and per-element, emoji (UTF-16 units), `CONCATENATE({"a";"b"},"x")`, `CONCATENATE(A1:A2,B1:B2)`, mismatched shapes, number/boolean in arrays; oracle values recorded.
- [ ] arrays.md entry status updated; implementation fixed if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
