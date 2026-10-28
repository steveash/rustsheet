---
id: T-2466
title: "Probe CELL('contents') in VSTACK: non-blank target, HSTACK, upper-case info type, beside an error argument"
milestone: M7
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for how far the `VSTACK(CELL("contents",ref),..)` `#VALUE!` (T-2457) reaches.

## Context
T-2457 matches the literal CELL call syntax; every probed target was blank. See docs/excel-semantics/information.md. Code: `is_cell_contents_call` in hstack_vstack.rs.

## Acceptance criteria
- [ ] Probe case with oracle values for: a non-blank target, HSTACK, an upper-case info type, CELL beside an error argument, and CELL wrapped in LET/IF; findings in information.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings (file separate tasks).
