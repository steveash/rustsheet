---
id: T-0094
title: "Classify poi formula cells into the M8 subset of decision 0146 (defined name, structured, 3-D or external reference) and report its counts"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0088]
---

## Goal
The eval report says how many scored poi cells fall in M8's "POI corpus subset" (decision 0146 item 2(b)), how many
of them are visibly off and which, and which crashed or errored poi cases hold a subset formula, so M8's exit row
(T-0096) can require ≥ 99.5% of the subset not visibly off.

## Context
- Decision 0146 (T-0088, PR #2144) item 2(b): the subset is "the scored poi cells whose formula (or array anchor's
  formula) contains a defined name, a structured reference, a 3-D reference or an external-workbook reference, as
  the engine's parser classifies it". Deviations leave the denominator (decision 0036); pending cases (decision
  0019's web-vs-desktop conflicts) are not scored; a crashed or errored case holding a subset formula fails the
  measure until it runs and is listed as `case!*`. Until the count exists, M8's measure is `n/a`.
- The 2026-10-28 numbers in decision 0146: poi 336 cases (205 pending, 10 crashes, 14 errors), 27,209 cells,
  95 visibly off over the whole suite. The report cannot tell today which of them are M8 cells.
- The parser's node kinds already separate these forms: `Node::DefinedNameKind`, `Node::StructuredReferenceKind`,
  `Node::SheetSpanKind` (3-D, `parser/sheet_span.rs`) and the external-workbook reference forms (T-0473's
  external-link cache). Reuse the parser and an existing walker where one fits (`parser/contains_function.rs`,
  `stringify::bound_variables::walk_names`); do not classify by regex over the formula text the way
  `formula_calls.rs` finds calls, since decision 0146 names the parser.
- Untrusted input (CLAUDE.md directive 5): poi workbooks are third-party files. The classifier must return a result
  for every formula (unparsable formula: not in the subset, counted in a separate `unparsed` count), never panic,
  and recurse no deeper than the parser's own nesting bound (or walk iteratively).
- This is the poi analogue of `function_scope.rs` (cells sorted by what their formula uses); put the counts on
  report.json as their own section, so T-0096 reads them without rescoring.

## Acceptance criteria
- [ ] One classifier function (engine or eval crate, next to the existing walkers) answers, for a formula text and
      its workbook's defined names, whether it holds each of the four forms; unit tests cover each form alone,
      a formula with none, a name that is a LET/LAMBDA parameter rather than a defined name (not in the subset),
      an unparsable formula, and a formula nested past the parser's depth bound (no panic, no stack overflow).
- [ ] report.json gains a poi M8-subset section: `scored`, `visibly_off`, `visibly_off_cells` (`case!Sheet!A1`),
      `deviations`, `unparsed`, and `blocked_cases` (crashed or errored cases that hold a subset formula, as
      `case!*`); absent (null) when poi was not run. report.md prints one line with the counts.
- [ ] `rustsheet-eval fetch` then `make eval` with poi fetched produces the section; the Outcome note records its
      numbers. Wall time of `make eval` grows by no more than 5% (measured and recorded).
- [ ] No baseline, gate or `scripts/` change; `make check` green; reviewers approve.

## Out of scope
- The M8 exit row and exit-owners support (T-0096).
- Fixing any poi cell, or filling pending poi cases.
