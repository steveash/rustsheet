---
id: T-3102
title: "Probe whitespace after the bang in an R1C1-mode formula (formulasR1C1) and what we do with S! R1C2 typed in A1 mode"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for whitespace after `!` in a formula entered in R1C1 mode (`S! R1C2`, via `formulasR1C1`), and what RustSheet does with `=S! R1C2` typed in A1 mode (Excel refuses it, 400 InvalidArgument); adjust the lexer if it differs.

## Context
T-2869 probed the A1-mode forms only (docs/excel-semantics/syntax.md, "drop after `!`"). Fidelity-review follow-up.

## Acceptance criteria
- [ ] Probe case(s) with oracle values; engine matches; syntax.md entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Tried: scoped the probe. Blocker found: no spec cell can enter R1C1-mode text today. `{"entered": ...}` is
written by the excel-graph backend as a `values` PATCH (tools/src/rustsheet_tools/spec2xlsx.py `entered_cells`
via docProps `rustsheet.entered.<n>`, tools/src/rustsheet_tools/oracle/backends.py line ~84, docs/architecture/oracle.md
"Typed input"), and `rg formulasR1C1` finds nothing in tools/ or crates/. The probe needs a new entry mode
(e.g. `{"entered_r1c1": "=S! R1C2"}` written as a `formulasR1C1` PATCH), plumbed through spec2xlsx.py, the
backend, `CellInput` in crates/rustsheet-eval/src/case.rs and the engine adapter (engine.rs ~491), plus a
decision on whether the engine adapter should enter it in R1C1 mode. That is a schema/design change across
Python and Rust, beyond a sonnet S task.
Next: add the entry mode (own spec/decision if it changes the case schema), then write the probes
(`=S! R1C2`, `=S!`+LF+`R1C2`, tab variants in R1C1 mode; `=S! R1C2` in A1 mode expecting 400), fill with
excel-graph (status OK today), update syntax.md "drop after `!`" and adjust the lexer if it differs.
