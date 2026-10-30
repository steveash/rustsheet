---
id: T-3193
title: "Unary sign runs cost no parser depth per sign, so Excel's 1,024-minus and 1,023-plus sign runs parse"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet loads and evaluates the unary sign runs Excel accepts in probes/t3172-minus-*,
-plus-* and -alt-* (255, 256 and up to 1,024 minus / 1,023 plus or alternating `-+` signs, stored
and typed), with untrusted formulas still bounded (directive 5), and refuses the next sign as Excel
does if that is cheap to match.

## Context
T-3189 pinned these eighteen cases whole in eval/suites/probes/deviations (decision 0168) because
the depth bound refuses them ("Formula is nested too deeply", `MAX_FORMULA_DEPTH` in
crates/rustsheet-engine/src/expressions/parser/depth.rs). `Parser::parse_signs` already reads a run
in a loop, but `Parser::finish_power` wraps one `UnaryKind`/`UnaryPlus` node per sign and calls
`grow` for each, so the tree (evaluator, printer, copy/drop) is as deep as the run. A run could be
folded into one node (the signs in written order, e.g. a count of minuses plus the kept pluses and
their whitespace runs) so its depth cost is constant, as operator chains are (T-0128, chain.rs).
Keep the coercion each sign implies (`--x` coerces `x` to a number) and the printed text. Excel's
caps (T-3172; syntax.md "Formula nesting depth cap"): 1,024 minus signs before a number, 1,023 plus
signs before text or 1,023 alternating `-+` signs before a reference, 1,025 / 1,024 refused. The
parenthesis case (t3172-paren-256) belongs with T-3191. This may need a spec first if the node
change is bigger than M.

## Acceptance criteria
- [ ] The eighteen sign-run cases load and match Excel; their whole-case lines leave
      probes/deviations (the stale report flags them) and the baseline records the passes.
- [ ] At-limit tests on a 1 MiB release stack (test_nesting_depth.rs) cover runs of 1,024 signs;
      syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bare parentheses (T-3191); function-call nesting (T-3097, T-3143).
