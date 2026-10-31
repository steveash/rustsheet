---
id: T-3283
title: "Typed =1**2 stores =100 in Excel: match Excel's reading of ** and of unprobed missing-) shapes in a typed formula"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula typed with `**` reads as Excel reads it: `=1**2` typed stores `=100` (value 100) in Excel.

## Context
probes/t3213-typed-parse-errors S!A5, B5, C5 (excel-graph): `=1**2` typed is not refused; FORMULATEXT
is `=100`. RustSheet refuses it as a parse error since T-3213 (docs/excel-semantics/syntax.md "A typed
formula that does not parse is refused"). Excel's rule is unknown: probe first (`=2**3`, `=1**`,
`=A1**2`, `=1***2`, `=1**2` stored) to tell an autocorrect of `**` from a number-literal reading
(`1E2`-like) before implementing. The same probe should pin the unprobed missing-`)` shapes T-3213
chose a rule for (`typed_input::open_parenthesis_is_a_call`): `=SUM(1+(2`, `=SUM((1`, `=LAMBDA(x,x)(1`,
`=SUM(SUM(1` (two open), and match them. Code: `Model::parse_cell_formula` (model.rs), typed_input.rs.

## Acceptance criteria
- [x] t3213-typed-parse-errors A5/B5/C5 pass, and a probe of more `**` shapes agrees.
- [x] The unprobed missing-`)` shapes are probed and match.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed first (excel-graph, 2026-10-31): probes/t3283-typed-double-star shows that typed `**` is a number's
exponent marker `E`, not a power: `=2**3` stores `=2000`, `=1.5**2` `=150`, `=2**-1` `=0.2`, and `**` where an
`E` could not stand (after a reference or `)`, before a space or a fraction, a second exponent, `***`) is refused.
probes/t3283-typed-missing-paren shows every shape with two `(` open (`=SUM(1+(2`, `=SUM((1`, `=SUM(SUM(1`,
`=ABS(-(1`, `=SUM(1,(2`, `=LAMBDA(x,x)(1`) is refused, which `open_parenthesis_is_a_call` already did.
The lexer reads `**` as `E` while parsing typed text (`Lexer::set_typed`, set by `Parser::set_typed`);
`consume_number` moved to lexer/number.rs (file-size gate). Stored `**` is unprobed and stays a parse error.
syntax.md has the new rule. Eval: probes known failures 2118 -> 2115 (t3213-typed-parse-errors A5, B5, C5),
two new cases fully passing, 0 regressions.
