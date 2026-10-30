---
id: T-3212
title: "Read Excel's bare R1C1 offsets (RC, R[-1]C, RC[2]) in R1C1 typed entry, with oracle probes"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3113]
---

## Goal
A spec cell typed in R1C1 mode (`{"entered_r1c1": ...}`, T-3113) with Excel's short R1C1 forms (`=RC`, `=R[-1]C`, `=RC[2]`, whole rows `=R2`, `=R[1]`, columns `=C3`, `=C[-1]`) reads as Excel reads it.

## Context
`rustsheet_engine::r1c1_formula_to_a1` (crates/rustsheet-engine/src/typed_input.rs, `TODO(T-3212)`) parses with the
lexer's `LexerMode::R1C1`, which is the engine's storage grammar: it always writes `R[n]C[n]`, and bare `R`/`C`/`RC`
are identifiers there (LAMBDA parameters such as `c` re-parse from storage, `is_valid_r1c1_identifier` in
expressions/lexer/mod.rs). So the converter refuses `=R[-1]C`, and the eval adapter leaves the cell blank. Fix it in
the converter (a typed-entry flag on the lexer, or a pre-pass) without changing how stored formulas re-parse.
Probe first: which short forms Excel accepts typed in R1C1 mode, and what a LET name `rc` does there; also
cell-shaped LET/LAMBDA names (`f1`, refused typed in A1 mode by decision 0103) and a structured reference typed in
R1C1 mode. The converter returns `None` for every parse failure, so the adapter blanks text the engine merely does
not support (or that passes a nesting/length limit) as if Excel refused it: return a distinct result for those and
fail the case instead (fidelity review of T-3113).

## Acceptance criteria
- [ ] Probe case(s) with `entered_r1c1` cells for each short form, filled by excel-graph; engine matches.
- [ ] Stored formulas with `r`/`c`/`rc` LAMBDA parameters still load and evaluate (existing tests).
- [ ] Findings in docs/excel-semantics/syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- R1C1 display (FORMULATEXT in R1C1 mode).
