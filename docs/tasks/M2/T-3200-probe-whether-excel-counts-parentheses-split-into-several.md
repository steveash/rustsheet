---
id: T-3200
title: "Probe whether Excel counts parentheses split into several runs against its 256 limit"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/syntax.md "Formula nesting depth cap" states, with excel-graph evidence,
whether Excel's 256-parenthesis limit counts all groups open at once or only an unbroken run of `(`.

## Context
T-3172 probed only an unbroken run around `1` (256 load, 257 refused). T-3191 (decision 0173) makes
RustSheet refuse more than 256 groups open at once whether the run is split or not, and records
split runs as unprobed. Shapes to probe with the `excel-probe` skill, stored and typed, at 256 and
257 groups in all (sidecars only for the ones Excel loads; refused inputs are dropped after the run,
as T-3172's were): `=( ` + run + `)` (whitespace split), `=(1+` + run + `)` (operand split),
`=(ABS(` + run + `))` (call split), and 128 + 129 groups split by `1+`. A formula Excel loads
should evaluate in a second cell (`ISNUMBER` or the value). If Excel counts only unbroken runs,
file an engine task to change `Parser::parse_parenthesis_run` (`open_groups`) to match.

## Acceptance criteria
- [ ] Probe cases for the shapes above are committed with Excel sidecars for every input Excel
      loads; syntax.md records the result and drops "unprobed" for the shapes covered.
- [ ] A follow-up engine task exists if RustSheet disagrees with Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up if needed); `(ABS(` levels (T-3197).
