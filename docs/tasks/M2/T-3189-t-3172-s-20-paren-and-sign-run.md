---
id: T-3189
title: "T-3172's 20 paren and sign-run probe cases count against M2 exit: lift the depth bound or pin them as decision-0036 deviations"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3172]
---

## Goal
The M2 exit row no longer counts T-3172's twenty probe cases as unowned known failures: each one
either matches Excel or is a pinned, accepted deviation (decision 0036) against the depth-bound
DEVIATION in docs/excel-semantics/syntax.md "Formula nesting depth cap".

## Context
T-3172 (excel-graph 2026-10-30) found that Excel accepts 256 bare nested parentheses, a run of
1,024 unary minus signs before a number, and runs of 1,023 unary plus signs before a text or
1,023 alternating `-+` signs before a reference, stored and typed alike, and refuses one more
(createSession 501 / Graph PATCH 400). RustSheet refuses every one of these cases as a whole
("Formula is nested too deeply", `MAX_FORMULA_DEPTH` = 256 in
`crates/rustsheet-engine/src/expressions/parser/depth.rs`). This is the same choice T-3165 makes
for T-3097's eight paren cases (lift the bound with the native stack still bounded, or pin the
cells as deviations); take T-3165's outcome and apply it here. Cases (each fails to load):
- probes/t3172-alt-1023-stored
- probes/t3172-alt-1023-typed
- probes/t3172-alt-255-stored
- probes/t3172-alt-255-typed
- probes/t3172-alt-256-stored
- probes/t3172-alt-256-typed
- probes/t3172-minus-1024-stored
- probes/t3172-minus-1024-typed
- probes/t3172-minus-255-stored
- probes/t3172-minus-255-typed
- probes/t3172-minus-256-stored
- probes/t3172-minus-256-typed
- probes/t3172-paren-256-stored
- probes/t3172-paren-256-typed
- probes/t3172-plus-1023-stored
- probes/t3172-plus-1023-typed
- probes/t3172-plus-255-stored
- probes/t3172-plus-255-typed
- probes/t3172-plus-256-stored
- probes/t3172-plus-256-typed

## Acceptance criteria
- [x] `make exit-owners MILESTONE=M2` does not list any of the cases above.
- [x] If the bound stays: each case is pinned in `eval/suites/probes/deviations` against the
      DEVIATION entry, which names the bound and Excel's limits as T-3172 probed them.
- [x] No regressions; `make check` green; reviewers approve.

## Out of scope
- Changing the probe cases or their sidecars; function-call nesting (T-3097, T-3143).

## Outcome
Kept the bound and pinned the twenty T-3172 cases whole, as T-3165 did for T-3097's paren cases
(decision 0168): one `*` line each in eval/suites/probes/deviations against syntax.md "Formula
nesting depth cap", whose DEVIATION already names `MAX_FORMULA_DEPTH` and Excel's T-3172 caps.
Every case is refused with "Formula is nested too deeply" (checked with `make eval-show`).
Eval delta: probes 20 cases `!EngineError` → pass (40 cells, all deviations); `make exit-owners
MILESTONE=M2` no longer lists them; no regressions. Follow-up: T-3193 (lift the bound for unary
sign runs); t3172-paren-256 goes with T-3191 (parentheses).
