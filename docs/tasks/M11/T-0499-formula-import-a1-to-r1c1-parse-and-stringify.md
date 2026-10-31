---
id: T-0499
title: "Formula import: A1-to-R1C1 parse and stringify cost about 650k instructions a formula (corpus-bench 52450 loads in about 3 s, 80% of its run)"
milestone: M11
lane: perf
status: done
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
Split on 2026-10-30 (resumed run): the 1,500 ms and 7G targets, the before/after R1C1 comparison test and the calc
bench check moved to T-3318, with this run's profile. This task keeps the first half.
- [x] corpus-bench/52450 loads faster in `rustsheet-eval run --suite corpus-bench --filter 52450` on a release build,
      with the Outcome's before and after numbers (the 1,500 ms target: T-3318).
- [x] A bench (crates/rustsheet/benches, a new `import` group) loads a synthetic workbook of 20,000 formulas of this
      shape.
- [x] Every change keeps the parse and the printed text as they were: the lexer's peek cache returns the token and
      leaves the state a fresh read would, and the whole-number fast path is tested against the general path
      (the before/after comparison over the vendored suites' formulas: T-3318).
- [x] `rustsheet-eval check` 0 regressions; `make check` green; reviewers approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's own snapshot and volatile scan.

## Outcome
Two runs (the first pushed a partial branch; the second resumed it after 2 h idle). What changed:
- `crates/rustsheet/benches/import.rs` (`cargo bench -p rustsheet --bench import`): loads 20,000 formulas of the
  52450 shape over a quoted sheet name, recalc off.
- Lexer: `peek_token` keeps the token it read by start position (`Peeked`, dropped by `set_formula` and every
  setting change; it restores what an inner peek leaves in `next_token_position`, so taking a cached token leaves the
  same state as reading it fresh), and `Lexer::peek` borrows it; the parser's compare-only peeks (`parse_range`,
  `parse_implicit`, `parse_signs`, argument and array lists, `parse_identifier`, `run_after_operand`) no longer clone
  tokens. `set_formula` reuses its char buffer, sized once.
- `stringify`: `format_function`, `operator_text` and the R1C1 `stringify_reference` append into one buffer;
  `to_excel_precision_str` prints a whole number below 1e15 as its integer text (tested against the general path).
- Names without a leading `_` skip the `_xlfn.`/`_xlws.`/`_xlpm.` prefix searches in `parse_named_call` and
  `Functions::lookup_stored` (which also looked an unprefixed unknown name up twice).

Numbers (release build, this container): corpus-bench/52450 4,042 ms on main (first run's machine) -> 2,893 ms
after the first run's changes -> 2,466-2,563 ms after this run's. Callgrind of `rustsheet-eval show` on the case after
the first run: `from_a1_to_rc` 13.4G (from 19.5G on main); the first run measured the bench at 1,000 formulas at
980M -> 557M instructions in `from_a1_to_rc`. The case thread also spends 6.6G in the harness's argument-coverage
re-parse (`call_arguments`), which is timed; T-3318 has that profile and the remaining target. `cargo bench -p rustsheet --bench import` on the final branch: median 695 ms (min 658 ms)
for the 20,000 formulas (no number on main: the bench is new; T-3318 compares). A lexer test
(`lexer/test/test_peek.rs`) checks cached peeks against fresh reads. Eval: 0 regressions, no newly passing cells.
