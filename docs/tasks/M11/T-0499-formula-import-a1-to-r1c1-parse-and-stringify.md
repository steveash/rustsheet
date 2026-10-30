---
id: T-0499
title: "Formula import: A1-to-R1C1 parse and stringify cost about 650k instructions a formula (corpus-bench 52450 loads in about 3 s, 80% of its run)"
milestone: M11
lane: perf
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Loading a workbook of tens of thousands of long formulas costs a small multiple of reading their XML. Today the
importer's A1-to-R1C1 pass (lex, parse, stringify) dominates such loads. The imported model, every formula's text and
every value stay identical.

## Context
Gap triage 2026-10-30 (opus refill). `rustsheet-eval run --suite corpus-bench` on origin/main c59497fc:
corpus-bench/52450/1_52450_input.xlsx takes 3,851 ms, the slowest corpus-bench case. It has 38,044 formulas and no
defined names. 34,770 of the formulas are 400-499 characters long, mostly normal (not shared) formulas such as
Sheet1!AA100:

    IF($E100<>"",IF(COUNTIFS('[1]XA Data Pull'!$J:$J,">="&AA$1,'[1]XA Data Pull'!$J:$J,"<="&AA$1+TIME(23,59,59),...)=0,"",COUNTIFS(...)),"")

Calculation is cheap (T-0477 made the linked-workbook criteria ranges `#VALUE!` without building arrays). Callgrind
(`valgrind --tool=callgrind ./target/release/rustsheet-eval show corpus-bench/52450/1_52450_input.xlsx`), 25.8G
instructions in total:
- `rustsheet_xlsx::import::load_from_xlsx_bytes_with_limits` 20.6G (80%). Of that,
  `import::worksheets::from_a1_to_rc` takes 19.5G over 30,306 calls, about 643k instructions a formula, roughly 1,500
  per character;
- inside it `Parser::parse` 12.2G and `stringify::to_rc_format` 5.8G (`format_function`, `stringify_reference` 2.4G);
- `Lexer::next_token` 8.1G inclusive (1.36G self), `consume_reference_a1` 0.55G, `consume_range_a1` 0.44G;
- allocation: malloc, free and realloc about 6.9G, `RawVecInner::finish_grow`/`reserve` 1.6G, `format_inner` and
  `fmt::write` 1.1G, `TokenType` drops 0.35G.

`from_a1_to_rc` (crates/rustsheet-xlsx/src/import/worksheets.rs) parses each A1 formula, adds implicit intersections,
prints it as R1C1, and `find_or_add_formula` dedupes the text. The engine then parses each distinct R1C1 text again.
T-0450 removed the last import hot spot (conditional formats) and found the parse itself small on pv26_001's formulas.
This file's long formulas with external-workbook sheet prefixes show the per-character cost, which is not small.

Profile first, then fix what the profile shows. Candidates, none measured:
- per-token `String` building in the lexer (quoted external sheet names `'[1]XA Data Pull'` are re-allocated at each of
  about ten references a formula);
- `format!` in `stringify` where a `write!` into one buffer would do;
- a cache keyed by the A1 text's R1C1 shape: identical relative formulas along a row or column, as here, could skip the
  parse, but only if the key is exactly what makes the outputs equal. Prove that, or skip it.

## Acceptance criteria
- [ ] corpus-bench/52450 takes at most 1,500 ms in `rustsheet-eval run --suite corpus-bench --filter 52450` on a release
      build (from 3,851 ms). The Outcome gives callgrind's `from_a1_to_rc` instruction count before (19.5G) and after;
      the target is at most 7G.
- [ ] A bench (crates/rustsheet/benches, a new `import` group or an existing one) loads a synthetic workbook of 20,000
      formulas of this shape. The calc benches do not slow by more than 2%.
- [ ] Every imported formula's R1C1 text is unchanged: a test compares `from_a1_to_rc` output before and after on
      the formulas of the vendored suites (fixtures, ironcalc-calc), or a property test over the parser's test
      corpus.
- [ ] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure; `make check` green; reviewers
      approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's own snapshot and volatile scan.
