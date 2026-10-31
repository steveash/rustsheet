---
id: T-3266
title: "Probe which unary signs count toward Excel's sign cap and how nested -( runs share it"
milestone: M2
lane: research
status: done
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

## Outcome
77 single-formula probes (stored and typed) filled from excel-graph on 2026-10-31. The 50 Excel accepted
are kept with sidecars. Of the 27 it refused, the four typed cases where ours disagrees are kept, since their
sidecar records the refusal; the rest were dropped after the run, as T-3172's were, and are recorded in
syntax.md. Fitted rule: the signs open at an operand (across nested `-(` runs) plus the
operand's cost may be at most 1,024. A number literal right after the signs costs 0; a reference,
`TRUE`, a string or a spaced number costs 1; a parenthesis run or a call costs 2. `-(` nests to the
256-parenthesis cap, which also confirms that split parenthesis runs count together. `MAX_SIGNS`
holds for literals, references and booleans. Ours admits one sign too many before `(` or a call,
and refuses 128+ levels of `-(` through `MAX_FORMULA_DEPTH`, so 16 cases are known failures, filed as
T-3338. Eval delta: 38 new passing cases (76 cells), 16 new known-failure cases, 0 regressions.
