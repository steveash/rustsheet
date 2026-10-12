---
id: T-1204
title: "Probe near-zero snapping inside conditional-formatting formulas on desktop Excel"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. Conditional-format rendering is not observable through excel-graph, which returns values only (T-1171).
The evidence needs desktop Excel. Unblock when `cd tools && uv run rustsheet-oracle status` shows
`excel-desktop OK` (2026-10-12: `n/a`).

## Goal
Desktop Excel evidence says whether a conditional-formatting rule formula snaps near-zero results, and whether a formula wholly in parentheses is exempt as in a cell.

## Context
T-1171 found the CF effect unobservable through excel-graph (values only). numbers.md "Near-zero snapping" marks the CF behavior UNVERIFIED. Probe `=(0.1+0.2-0.3)=0` against `=0.1+0.2-0.3=0` rules and compare the rendered fill; the engine keeps the grouped root via `user_formula_to_internal`.

## Acceptance criteria
- [ ] Desktop result recorded in numbers.md with the probe workbook; engine mismatches baselined and follow-up filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
