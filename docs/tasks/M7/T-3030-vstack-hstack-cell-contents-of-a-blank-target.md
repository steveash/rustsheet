---
id: T-3030
title: "VSTACK/HSTACK: CELL('contents') of a blank target is #VALUE!, non-blank targets stack (T-2466 evidence)"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`VSTACK`/`HSTACK` match Excel for `CELL("contents", ref)` arguments: a blank target makes the stack `#VALUE!` (also through `LET`, `IF` and an info type read from a cell), a non-blank target stacks its value.

## Context
T-2466 probed the T-2457 rule (docs/excel-semantics/information.md, CELL("contents") in VSTACK). Evidence:
probes/t2466-cell-contents-stack (27 cell failures today). Ours keys on the call's syntax
(`is_cell_contents_call` in crates/rustsheet-engine/src/functions/lookup_and_reference/hstack_vstack.rs, applied for VSTACK only),
so it errors on non-blank targets (A1, A11, A21, A181, A191, A211), misses the LET/IF/cell-info-type forms (A111, A121, A151),
and HSTACK has no rule (A41). Separately, `CELL("contents",Data!B1)` (B1=7) nested in an argument of HSTACK, TOCOL, `+0` or
LET/IF inside VSTACK evaluates to 0 or blank instead of 7 (A51, A131, A141, A221, A231, A241):
find why before choosing the design (the stack argument may be evaluated in a context where CELL loses its reference).
The rule needs the blank-target CELL("contents") result to be distinguishable as a value (it is 0 alone, "|" with &"|"), so it can
survive LET/IF; a design that only extends the syntax match is not enough.
Before choosing that design, probe `VSTACK(CELL("contents",Data!C11)+0,1)`, `VSTACK(CELL("contents",Data!C11)&"",1)` and
`TOCOL(CELL("contents",Data!C11))` (blank target): whether the blank-target value survives arithmetic, concatenation and
other shapers is unprobed (T-2466 review).

## Acceptance criteria
- [ ] probes/t2466-cell-contents-stack passes in full, including the nested non-blank cells A51, A131, A141, A221, A231 and A241; probes/t2414-gen-m7-unexplained keeps passing.
- [ ] information.md's "Ours" paragraph updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TOCOL/WRAPROWS and other shapers with a blank target (unprobed; probe first if needed).
