---
id: T-3191
title: "Nested parentheses cost less parser stack, so Excel's 255 bare parentheses and 65 (ABS( levels parse"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet loads and evaluates the formulas Excel accepts in probes/t3097-nesting-paren-128, -255,
-abs-64 and -abs-65 (stored and typed), with untrusted formulas still bounded (directive 5).

## Context
T-3165 pinned these eight cases whole in eval/suites/probes/deviations (decision 0168) because the
parser refuses them ("Formula is nested too deeply", `MAX_FORMULA_DEPTH` in
crates/rustsheet-engine/src/expressions/parser/depth.rs). Measured for T-3165 (release, 1 MiB
thread): the parser alone overflows at about 292 nested parentheses (about 3.5 KB per level), 180
`ABS(` and 112 `(ABS(`, because every `(` passes through the whole precedence chain
(`parse_primary` → `parse_parenthesized` → `parse_expr` → five `parse_chain` levels → `parse_power`
→ … → `parse_primary`). A directly nested group (`((x))`) could skip the chain, and a `Group` node
(one node for many levels, `levels: u8`, saturating today) could cost less than a full operand
level, so the bound covers what really uses the stack. Watch `Group.levels` past 255 (saturating
add would drop a parenthesis from the printed text). Excel's own limit on bare
parentheses is 256 (T-3172; syntax.md "Formula nesting depth cap"); T-3189 owns the T-3172 cases. This may
need a spec first if the parser change is bigger than M.

## Acceptance criteria
- [ ] The eight cases load and match Excel; their whole-case lines leave probes/deviations (the
      stale report flags them) and the baseline records the passes.
- [ ] At-limit tests on a 1 MiB release stack (test_nesting_depth.rs) and a 2 MiB debug stack
      (test_nested_formula_stack.rs) cover the new bound; syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
