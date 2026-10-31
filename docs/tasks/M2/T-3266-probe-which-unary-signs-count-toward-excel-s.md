---
id: T-3266
title: "Probe which unary signs count toward Excel's sign cap and how nested -( runs share it"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence that settles which unary signs count toward Excel's sign cap, and whether signs
in nested runs (`=-(-(...))`) share one budget, so `MAX_SIGNS` either is confirmed or gets a
follow-up to match.

## Context
T-3193 lifted the depth bound on sign runs and capped them at `MAX_SIGNS` (1,023 open on one
parse path, a minus written directly before a number literal not counted; depth.rs,
`Parser::parse_signs`). That rule fits the three caps T-3172 bisected (probes/t3172-minus-*, -plus-*,
-alt-*: 1,024 minus before `1`, 1,023 plus before `"a"`, 1,023 `-+` before `C1`), but a cap that
depends on the operand's type would fit them too (docs/excel-semantics/syntax.md "Formula nesting
depth cap"). Shapes that tell the two apart: 1,024 / 1,025 minus before `(1)` and before `- 1`
(spaced), 1,024 plus before `1`, 1,024 minus before `TRUE` and before `C1`. Nested runs: `-(`
repeated around a run, e.g. 200 levels of `-(` with an 824- and 825-sign run inside, and mixed
`-(` nesting against the 256-parenthesis cap. Use the excel-probe skill, stored and typed.

## Acceptance criteria
- [ ] Probe cases for each shape above (accepted ones with sidecars; refusals recorded in
      syntax.md as T-3172 did) and the syntax.md entry states the rule the evidence supports.
- [ ] If RustSheet disagrees, a follow-up engine task names the cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task if needed).
