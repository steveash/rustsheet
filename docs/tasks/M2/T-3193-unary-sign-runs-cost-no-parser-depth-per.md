---
id: T-3193
title: "Unary sign runs cost no parser depth per sign, so Excel's 1,024-minus and 1,023-plus sign runs parse"
milestone: M2
lane: engine
status: done
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
- [x] The eighteen sign-run cases load and match Excel; their whole-case lines leave
      probes/deviations (the stale report flags them) and the baseline records the passes.
- [x] At-limit tests on a 1 MiB release stack (test_nesting_depth.rs) cover runs of 1,024 signs;
      syntax.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bare parentheses (T-3191); function-call nesting (T-3097, T-3143).

## Notes
- 2026-10-30: resumed by an opus run (the claim had been idle since 08:32 with no further commit).

## Outcome
- A unary sign no longer costs a level of `MAX_FORMULA_DEPTH`; instead `MAX_SIGNS` (1,023) bounds
  the signs open on one parse path, a minus written directly before a number literal not counted,
  which matches every T-3172 cap (1,024 minus before `1`, 1,023 plus before `"a"`, 1,023 `-+`
  before `C1`; one more is "Formula is nested too deeply"). Nested runs (`=-(-(...))`) share the
  budget (unprobed in Excel; syntax.md says so). No new node: the evaluator already walked a run in
  a loop (`apply_unary_chain`); the printer now does too: `stringify::wrapped_text` prints a run
  of signs, written parentheses and whitespace in one loop (it overflowed a 1 MiB release stack at
  1,023 spaced signs, and a 2 MiB debug stack at 127 levels of `- ( `, which now nests as deep as
  `(`). `group_text` and `Spacing::around` are gone with it.
- Tests on a 1 MiB release stack: a 1,023-sign run (bare, spaced, line-fed) under the deepest
  `(1+` nesting the depth bound admits parses, evaluates, prints, survives a row insert and a
  sheet rename; one more sign, alone or across a `-(` run, is too deep. Measured headroom: the
  same test overflows at 512 KiB (in the derived `Clone`), so about 2x.
- The eighteen t3172 sign-run lines left probes/deviations; the cases now load and match Excel.
- Eval delta: probes 18 cases (36 cells) move from whole-case deviation pins to bit-exact passes
  (probes/deviations lines 41 -> 23; the probes suite now scores 27 deviation cells); known failures unchanged (2672 -> 2672), no regressions.
- Follow-ups: T-3252 (probe which signs count and how nested `-(` runs share the cap), T-3253
  (wider stack margin: iterative Clone/Drop of sign runs).
