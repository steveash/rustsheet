---
id: T-0168
title: "Check autofill output for tiny/huge steps uses plain decimals like Excel"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Autofill/sequence output for very small or large steps prints numbers the way Excel does (no exponent notation where Excel shows plain decimals, or vice versa).

## Context
Left over from T-0150/T-0143: `to_excel_precision_str` (used by user_model/sequence_detector.rs) may print exponent notation where `to_string` printed plain decimals. Needs Excel evidence (excel-probe skill) for fill series with steps like 1E-10 and 1E+20.

## Acceptance criteria
- [ ] A probe or unit test records Excel's fill output for tiny/huge steps; the engine matches or the quirk is documented in numbers.md.
- [ ] `make check` green; reviewers approve.

- [ ] A probe cell `="123456789012345678"+0-123456789012345000` is oracle-filled in probes/text-number-15-digits.

## Out of scope
-
