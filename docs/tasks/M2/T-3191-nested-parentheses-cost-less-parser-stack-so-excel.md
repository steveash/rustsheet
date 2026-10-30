---
id: T-3191
title: "Nested parentheses cost less parser stack, so Excel's 255 bare parentheses and 65 (ABS( levels parse"
milestone: M2
lane: engine
status: done
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
parentheses is 256 (T-3172; syntax.md "Formula nesting depth cap"); T-3189 pinned the T-3172 cases whole; t3172-paren-256 leaves the deviations with these eight. This may
need a spec first if the parser change is bigger than M.

## Acceptance criteria
Split (2026-10-30, T-3191): the bare-parenthesis cases stay here; the four `(ABS(` cases moved to
T-3197, because a `(` before a call still recurses and needs a separate depth change.
- [x] The bare-parenthesis cases (t3097-nesting-paren-128, -255 and t3172-paren-256, stored and
      typed) load and match Excel; their whole-case lines leave probes/deviations and the baseline
      records the passes.
- [x] At-limit tests on a 1 MiB release stack (test_nesting_depth.rs) and a 2 MiB debug stack
      (test_nested_formula_stack.rs) cover the new bound; syntax.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- t3097-nesting-paren-abs-64 and -65 (T-3197); unary sign runs (T-3193).

## Outcome
A run of directly nested parentheses is parsed in a loop (`Parser::parse_parenthesis_run`): the
innermost group first, then each enclosing one with the group inside handed to its content as its
first operand (`Parser::pending_operand`, returned by `parse_primary`; `parse_power` and
`parse_implicit` read no sign, run or `@` before it). A group that is exactly the group inside merges
into it and adds no depth; more than 256 groups open at once, in one run or several
(`MAX_PARENTHESES`, Excel's limit for a run; split runs unprobed), is too deep.
`Group.levels` is now `u16`. Decision 0173. Eval delta: probes t3097-nesting-paren-128/-255 and
t3172-paren-256 (stored and typed, 6 cases) go from whole-case deviations to bit-exact passes (12
cells); no regressions. Follow-up: T-3197 (`(ABS(` levels).
