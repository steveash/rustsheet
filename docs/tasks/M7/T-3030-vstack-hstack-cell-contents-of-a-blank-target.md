---
id: T-3030
title: "VSTACK/HSTACK: CELL('contents') of a blank target is #VALUE!, non-blank targets stack (T-2466 evidence)"
milestone: M7
lane: functions-lookup
status: done
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
      Partly met: every cell passes except the blank-target LET/IF/info-type cells A111, A121, A151, which moved to T-3053 (see Outcome).
- [x] information.md's "Ours" paragraph updated.
- [x] `make check` green; reviewers approve (round 2 after the fidelity review asked for a complete re-fill).

## Out of scope
- TOCOL/WRAPROWS and other shapers with a blank target (unprobed; probe first if needed).

## Outcome
- **Root cause of the nested-0 cells.** Ours read `CELL("contents"|"type", Data!X)` on the formula's own sheet, per
  T-1230/T-1433. Excel for the web (excel-graph) now reads the named sheet everywhere: on 2026-10-28 the T-1230/T-1433
  probes were re-filled twice with identical values, and new probes cover sheet order (t3030-cell-first-sheet) and
  nesting (t3030-cell-contents-nested-sheet). CELL now reads the named sheet. Every case with a cross-sheet
  `CELL("contents"|"type")` whose sidecar predated 2026-10-28 was re-filled from excel-graph: 201 sidecars, in which only
  CELL cells and their spills changed (information.md, decision 0153).
- **Blank-target stack rule.** VSTACK and HSTACK reject a literal `CELL("contents", ref)` argument only when the target
  is blank. probes/t3030-cell-contents-blank-shapers shows that the blank-target result is an empty value: array-taking
  functions reject it even through LET/IF/IFERROR/CHOOSE. Ours cannot tell that value from a blank reference read
  through those wrappers, so t2466 A111, A121, A151 and the other array-taking callers went to T-3053 (size M, a
  CalcResult design question).
- **Eval delta.** probes: 24 fixed and 3 new cases. gen-m9: 3 fixed (one of them the MODE.SNGL cell
  that the stale sidecar had masked). One IMTAN cell (gen-m9 756a9f7bc3a06bcc A338)
  was unmasked by the re-fill and allowed by decision 0153; it moved to T-3054. Visibly-off in-scope cells went from
  2572 to 2494.
