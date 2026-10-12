---
id: T-1299
title: "Desktop Excel: tiny number in conditional-format rule values; audit parse_finite_number sites"
milestone: M14
lane: engine
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. Conditional-format rule evaluation is not observable through excel-graph, which returns values only.
The evidence needs desktop Excel. Unblock when `cd tools && uv run rustsheet-oracle status` shows
`excel-desktop OK` (2026-10-12: `n/a`).

## Goal
Conditional-format rule values with tiny numbers (`Cell value > 1e-310`, `1e-400`) behave as desktop Excel does, with evidence; the remaining `parse_finite_number` readers are audited.

## Context
T-1100 showed typed cell entry follows the T-0154 tiny-number rule (probes/typed-number-tiny). Conditional-format rule values are not observable through excel-graph (cell values only), so they need desktop Excel (M14, Phase 4, decision 0039). `parse_finite_number` in conditional_formatting.rs still reads `1e-400` as 0; `sequence_detector.rs` and `functions/criteria.rs` also still call it (audit whether intentional). See numbers.md "Number text of 1E+308 or more is not a number".

## Acceptance criteria
- [ ] Desktop-Excel evidence for a conditional-format rule `Cell value > 1e-310` and `1e-400`; numbers.md records it.
- [ ] The engine follows it, reusing `parse_excel_number`; the other `parse_finite_number` sites are fixed or commented as intentionally different.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
