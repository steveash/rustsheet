---
id: T-2856
title: "Probe: stop-if-true between conditional-format rules of different priorities"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how a "Stop If True" conditional-format rule combines with rules of higher
and lower priority over the same cell, recorded in docs/excel-semantics/ and matched by the engine.

## Context
`update_cf_cache` (conditional_formatting.rs) gives a stop-if-true rule's match precedence over the
results written before it: rules run highest priority first, so it discards the formats of rules
*above* it, while rules below it still add theirs. Excel's documented meaning is the reverse: once a
stop-if-true rule matches, no lower-priority rule applies. No probe or excel-semantics entry backs
either. Found reviewing T-0476, which keeps the behavior as it is. Formatting is not a cell value,
so the probe needs applied formats (desktop Excel, `DisplayFormat`), hence M14.

## Acceptance criteria
- [ ] A probe workbook with stop-if-true rules above, between and below other rules over the same
      cells, and the formats Excel applies, cited in docs/excel-semantics/.
- [ ] The engine matches, or a follow-up task is filed.
- [ ] `make check` green; reviewers approve.

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. The evidence needs desktop Excel. Unblock when `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` (2026-10-30: `excel-desktop  n/a`).

## Out of scope
- Rules over different cells.
