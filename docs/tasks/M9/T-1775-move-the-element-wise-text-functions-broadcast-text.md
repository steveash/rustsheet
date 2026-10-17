---
id: T-1775
title: "Move the element-wise text functions (broadcast_text, TextArg) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1771]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The element-wise text functions lift through the SPEC-0011 driver, with no result change. `TextArg` and `broadcast_text` are deleted, and the driver's size bound and text budget cover them.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Migration of the existing helpers".
- `broadcast_text` has 17 call sites: LEFT, RIGHT, MID, FIND/SEARCH, LEN, REPT, SUBSTITUTE, REPLACE, EXACT, TEXT, CODE/UNICODE/CHAR/UNICHAR, FILTERXML, BAHTTEXT, CONCATENATE, DATEVALUE/TIMEVALUE text.
- They must keep these rules:
  - The left-to-right first-failure rule (T-1022, arrays.md "Errors broadcast through `&`, comparisons, POWER and the element-wise functions"; probes/error-precedence-text).
  - REPLACE, MID and FIND coerce every argument before any range check (T-1056).
  - CONCATENATE's scalar-error short cut (arrays.md "CONCATENATE with a scalar error argument returns that error").
  - The 64 MiB text budget (T-0232), which moves into the driver.
- T-1771 lands the size bound first.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite. probes/error-precedence-text, error-arg-vs-range-check and concatenate-limit-spill keep passing.
- [ ] `rg -n "fn broadcast_text|enum TextArg" crates/rustsheet-engine/src` finds nothing. `scripts/check_duplication.py` does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New text lifts or changes in text semantics. TEXTSPLIT, TEXTBEFORE and TEXTAFTER delimiters (T-1271).
