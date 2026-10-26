---
id: T-0481
title: "A stored formula's typographic quotes parse as an undefined name: #NAME?, not #ERROR! (corpus-bench/48354 D2:D6)"
milestone: M11
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A formula read from a workbook that uses typographic quotes (U+201C `“` / U+201D `”`) where a string literal
was meant evaluates to `#NAME?`, as Excel does, instead of the engine's parse-error `#ERROR!`: Excel reads
`“D”` as a name, finds it undefined and answers `#NAME?`. The rule to implement is which characters Excel's
lexer accepts in a name token; the probe under Context pins it before any code change.

## Context
Gap triage 2026-10-26 (external suites fetched). `corpus-bench/48354/1_48354_input.xlsx` is a whole-case known
failure (`F corpus-bench/48354/1_48354_input.xlsx *` in eval/baseline/corpus-bench.txt; 5 of 5 scored cells
fail) and no open task owns it. Sheet1 D2:D6 each store (xl/worksheets/sheet1.xml, `<f t="array" ref="D2">`):

```
IF(B2="domestic",“D”,IF(B2="international",“I”,“N/A”))
```

with real U+201C/U+201D characters around `D`, `I` and `N/A` (an app or a paste turned the ASCII quotes into
curly ones). Excel's answer is `#NAME?`: both the workbook's own cached value (`<v>#NAME?</v>`) and the
excel-graph sidecar agree, so this is not a web-only value. Ours is `#ERROR!` (the parser refuses the token).
Excel therefore reads `“D”` as a *name* — its name lexer accepts these characters — and `#NAME?` is the
undefined-name result, exactly as docs/excel-semantics/references.md "Undefined names may contain backslashes"
records for `HSI\10_1\…` (poi/46535.xlsx Others!L12). The engine's lexer is
crates/rustsheet-engine/src/expressions/lexer/ (name characters) with the parser in
crates/rustsheet-engine/src/expressions/parser/mod.rs; `#ERROR!` is what a parse failure becomes.

Unprobed, and what makes this opus: which characters Excel accepts, and whether a *typed* formula behaves the
same as a stored one (the backslash entry has the same open edge, and T-1089 shows the stored and typed paths
can need different rules). Probe first with the `excel-probe` skill (excel-graph, tag M11), both stored and,
where the oracle can enter them, typed:
`=“D”`, `=“D”&"x"`, `=IF(TRUE,“D”,“I”)`, `=“N/A”`, `=€`, `=café`, `=ü1`, `=a“b`, `="x"“y"`, and
`=SUM(“A1”)`; also whether `FORMULATEXT` of the stored D2 keeps the curly quotes, and whether a name
*defined* as `“D”` can be referenced (`ISREF`/value) — that decides whether these tokens are ordinary names
or only an error shape. Record the rule in references.md beside the backslash entry, then implement the
accepting cases.

## Acceptance criteria
- [x] A probe case with excel-graph values for the forms above; docs/excel-semantics/references.md states the
      rule (VERIFIED) and cites corpus-bench/48354 D2:D6 with the probe cells.
- [x] Sheet1!D2:D6 of corpus-bench/48354 are `#NAME?`, so the `F corpus-bench/48354/1_48354_input.xlsx *` line
      disappears from eval/baseline/corpus-bench.txt, with no new F line in any baseline.
- [x] A parser/lexer unit test for a name token with U+201C/U+201D (and whatever else the probe accepts) that
      resolves to `#NAME?` when undefined, plus a test that a well-formed string literal is unaffected.
- [x] An xlsx round-trip test: the stored formula keeps its characters on export (the formula text is not
      rewritten).
- [x] `make check` green; reviewers approve.

## Out of scope
- Bare newer function names loading as `#NAME?` (T-1089).
- Any attempt to "repair" the formula into a string literal: Excel does not, and Excel is the spec.
- The other unowned corpus-bench cases (T-0480, T-0482).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong error (none is a last-digit difference), so the task is
in the campaign at the `semantics` stage. Its target is that every cell it owns has Excel's error value; the
second acceptance criterion states it.

## Outcome
Probes t0481-curly-quote-names A1-A10 and -b A1-A2 (excel-graph, stored formulas) show a curly-quoted token is an
ordinary undefined name: `#NAME?` when evaluated, `IF(FALSE,“D”,1)` = 1, `ISREF` FALSE, `ERROR.TYPE` 5,
FORMULATEXT keeps the quotes, `“A1”` is a name. The lexer now reads U+201C/U+201D as name characters
(`is_typographic_quote`, `is_quoted_name`); references.md records the rule. Engine and xlsx round-trip tests added.
Eval: corpus-bench known failures 6 -> 5 (48354, 5 cells), 0 regressions; 12 new probe cells pass.
Unprobed and recorded as such: typed formulas, defined names with these characters, `€`, a quote mid-name or after
a string (probes/t0481-curly-quote-names-c is committed pending: the oracle run timed out before answering it).
