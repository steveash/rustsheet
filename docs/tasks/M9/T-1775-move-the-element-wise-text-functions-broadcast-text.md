---
id: T-1775
title: "Move the element-wise text functions (broadcast_text, TextArg) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1771, T-1773, T-1785, T-1774]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The element-wise text functions lift through the SPEC-0011 driver, with no result change. `TextArg` and `broadcast_text` are deleted, and the driver's size bound and text budget cover them. This is the last migration task, and it proves that no ad-hoc lift remains.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Today's lifting code" and Test plan "No ad-hoc lift remains".
- `broadcast_text` has 17 call sites: LEFT, RIGHT, MID, FIND/SEARCH, LEN, REPT, SUBSTITUTE, REPLACE, EXACT, TEXT, CODE/UNICODE/CHAR/UNICHAR, FILTERXML, BAHTTEXT, CONCATENATE, DATEVALUE/TIMEVALUE text.
- They must keep these rules:
  - the left-to-right first-failure rule (T-1022, arrays.md "Errors broadcast through `&`, comparisons, POWER and the element-wise functions"; probes/error-precedence-text);
  - REPLACE, MID and FIND coerce every argument before any range check (T-1056);
  - CONCATENATE's scalar-error short cut (arrays.md "CONCATENATE with a scalar error argument returns that error");
  - T-1771's ordering: the array-size check comes first (`#ERROR!`), and the 64 MiB text budget's error stays `#VALUE!`. The budget moves into the driver.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite. probes/error-precedence-text, error-arg-vs-range-check and concatenate-limit-spill keep passing, as do T-1771's size and budget tests.
- [ ] crates/rustsheet/benches/text.rs runs on main and on the branch in release and is within 5%; the numbers are in the Outcome.
- [ ] SPEC-0011's final grep check:
  - `rg -n "fn lift_numbers|fn lift_number_operands|fn lift_with_defaults|macro_rules! single_number_fn|fn lifted_arg|fn parity_predicate|fn broadcast_text|enum TextArg|fn lift_with_cost|fn is_lifted|enum CriteriaArg" crates/rustsheet-engine/src` finds nothing.
  - `rg -n "bcast_get|bcast_idx" crates/rustsheet-engine/src` finds only functions/lift/, arithmetic.rs and the spec's non-goal rows (IF, INDEX, GROUPBY/PIVOTBY).

  Both outputs are in the Outcome.
- [ ] `scripts/check_duplication.py` does not rise, and any drop is recorded with `--update`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New text lifts or changes in text semantics. TEXTSPLIT, TEXTBEFORE and TEXTAFTER delimiters (T-1271).
