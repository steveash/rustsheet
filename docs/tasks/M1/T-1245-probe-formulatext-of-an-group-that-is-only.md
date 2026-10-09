---
id: T-1245
title: "Probe FORMULATEXT of an @ group that is only the leftmost part of the operand: nested, range-corner and multi-operand shapes (follow-up of T-1200)"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT for an `@` group that is only the leftmost part of the operand is probed for the shapes T-1200 extrapolated, and the engine matches.

## Context
T-1200 matched probes/at-group-not-whole-operand E30:E32 and extrapolated the rest (references.md "@ operand", UNVERIFIED list; characterization test `at_operand_group_shapes_no_probe_covers_are_characterized`). Probe (excel-probe skill): `=@((A1)):B1 C1`, `=@(A1):B1:C1 D1`, `=@(A1:B1):C1 D1`, `=@(A1 B1) C1 D1`, `=@((A1 B1) C1) D1`, `=@(A1):(B1) C1`, `=@(A1):B1` and `=@(A1 B1):C1`. Code: `lead_group_text`, `LeadGroup` (parser/reference_operators.rs, parser/mod.rs).

## Acceptance criteria
- [ ] Probe case with oracle values for every form above; references.md records the result; the characterization test is replaced by oracle values and any mismatch is fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
