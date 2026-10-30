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
Probe first: which short forms Excel accepts typed in R1C1 mode, and what a LET name `rc` does there.

## Acceptance criteria
- [ ] Probe case(s) with `entered_r1c1` cells for each short form, filled by excel-graph; engine matches.
- [ ] Stored formulas with `r`/`c`/`rc` LAMBDA parameters still load and evaluate (existing tests).
- [ ] Findings in docs/excel-semantics/syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- R1C1 display (FORMULATEXT in R1C1 mode).
