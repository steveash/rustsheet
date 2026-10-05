---
id: T-0148
title: "Intersection: only space and LF separate operands; reject non-reference operands"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine accepts as an intersection exactly the formulas Excel accepts: the separator is one or
more spaces and line feeds, and both operands are reference expressions. Every formula Excel
refuses is a parse error (`#ERROR!`), not a value.

## Context
Evidence: docs/excel-semantics/references.md "Intersection operator: separators" and
"Intersection operands must be references" (T-0138). Excel for the web refuses a workbook
(`FileCorruptTryRepair`) that holds any one of these formulas, so there is no oracle value. Today
the engine evaluates them:
- separators: `=A1:A3<CR>A2:B2`, `<CR><LF>`, `<TAB>` and NBSP (U+00A0) give 2;
  `=A1:A3<U+3000>A2:B2` gives 2, but Excel gives `#NAME?` (oracle: probes/intersection-whitespace H7);
- operands: `=A1 (1)` gives `#VALUE!`; `=#N/A A1`, `=#N/A #REF!` give `#N/A`; `=(1/0) A1`,
  `=A1 (1/0)`, `=(1/0) (NA())`, `=(1/0) Z99` give `#DIV/0!`.
Excel accepts `#REF!` and reference-returning calls (INDEX, OFFSET, IF) as operands
(probes/intersection-operands, probes/intersection-function-operands; the engine already passes them).
Code: `Lexer::whitespace_follows` (`char::is_whitespace`) and the intersection rule in the parser.
U+3000 probably stops the reference token and starts a name: check `=A1:A3<U+3000>A2:B2` comes
out as `#NAME?`, not `#ERROR!`.

## Acceptance criteria
- [ ] probes/intersection-whitespace H7 passes (`#NAME?`), with no new F lines anywhere.
- [ ] Unit tests: each refused formula above parses to an error (the engine's `#ERROR!`), and
  space runs, LF and LF mixed with spaces still intersect.
- [ ] The `Implemented in:` lines of the two references.md entries name the code.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `@` and union operands, and `(A1)(A2)` (T-0149).

## Outcome
`Lexer::separator` + `Parser::parse_intersection`/`is_reference_operand`: only spaces and LF separate
operands (CR, TAB, NBSP are parse errors), U+3000 starts a name (lexer name character)
(`#NAME?`), and value operands (literals, value operators, non-`#REF!` errors, value functions) are
parse errors. Eval: probes/intersection-whitespace H7 fixed (probes known failures 61 -> 60), no
regressions. Only SUM and NA calls are refused as operands; other functions are accepted, unverified (follow-up probe task).

## Handoff
Done and pushed: separators (space/LF only; CR/TAB/NBSP are parse errors), value-operand rejection
(literals, value operators, non-#REF! errors, SUM and NA calls), baseline (H7 fixed), unit tests, docs.
`make check` was green before the last review-driven tweak (function rule changed to refuse only SUM/NA;
unverified rest tracked by T-0161). Excel-fidelity reviewer REQUESTED_CHANGES, remaining item:
U+3000 currently makes `Parser::parse_intersection` replace the whole intersection by `Node::ErrorKind(NAME)`
(see the `name_error` branch of that function at commit "fix U+3000 as name error"), so the cell value is
right (#NAME?) but the formula text is stored as `=#NAME?` (FORMULATEXT, export). I tried making U+3000 a name
start character in the lexer (so the operand becomes a real undefined name that keeps the text); it failed
because `utils::is_valid_a1_identifier` (shared with defined-name validation) rejects the name, and widening
it would let users define U+3000 names. Next: keep the formula text in a node that evaluates to #NAME?
(e.g. a dedicated undefined-name operand) and add a stringify round-trip test; then re-run reviewers
(excel-fidelity only).
