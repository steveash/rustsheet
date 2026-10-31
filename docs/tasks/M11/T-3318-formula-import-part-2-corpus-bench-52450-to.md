---
id: T-3318
title: "Formula import, part 2: corpus-bench 52450 to 1,500 ms (stringify and legacy-@ pass, argument-coverage re-parse)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0499]
---

## Goal
corpus-bench/52450 (38,044 long formulas) runs in at most 1,500 ms in `rustsheet-eval run --suite corpus-bench
--filter 52450` on a release build, with every imported formula's R1C1 text and every value unchanged. This is the
target T-0499 set and did not reach; T-0499 took the case from about 4,000 ms to about 2,500 ms.

## Context
Split from T-0499 (see its Outcome for what landed and the numbers). Callgrind of `rustsheet-eval show
corpus-bench/52450/1_52450_input.xlsx` on T-0499's branch before its last commit (27.6G instructions in all; the timed
case thread is 22.4G of it, the rest is the harness's snapshot and volatile scan, which are not timed):
- `import::worksheets::from_a1_to_rc` 13.4G over 36,640 calls: `Parser::parse` 7.7G, `stringify::to_rc_format` 3.8G
  (of which `number_format::to_precision_str` 0.9G, removed by T-0499's whole-number fast path),
  `add_implicit_intersection` 1.5G (`add_root_legacy_intersection`; `rewrite_as_unspaced`, `automatic_at` clones).
- `call_arguments` (crates/rustsheet-engine/src/call_arguments.rs) 6.6G: the harness's argument-coverage census
  (`crates/rustsheet-eval/src/argument_coverage.rs`, `observe`) parses each Excel-evidence formula cell's A1 text
  again, with a new parser per cell, 31,729 parses. It runs in the timed case thread, so it is part of the 1,500 ms.
  It was not in T-0499's original profile (25.8G in all, 20.6G of it the load).
- Throughout: malloc/free about 20%; `TokenType` clones from `peek_token` in lists.rs (LET names) and lambda.rs;
  `Functions::lookup` uppercases the name and compares it with every function name (linear).

Candidates, none measured:
- argument coverage: parse each distinct formula once per case (a cache keyed by the text and the cell offset
  does not help: relative formulas differ per row), or read the shapes from the engine's parsed R1C1 trees per
  shared formula, resolving relative references per cell;
- `stringify` writing into one `&mut String` instead of returning a `String` per node;
- the legacy-`@` pass skipping `rewrite_as_unspaced`'s clones when no node can take an `@`;
- the R1C1-shape cache of T-0499's Context, only with a proof that the key decides the output.

## Acceptance criteria
- [ ] corpus-bench/52450 takes at most 1,500 ms as above: not met, about 2,000 ms; the target moves to T-3326 (see
      Outcome). Met: the Outcome gives callgrind's `from_a1_to_rc` count before and after (10.7G -> 6.2G, target at
      most 7G) and the case thread's split.
- [x] Every imported formula's R1C1 text is unchanged: a test compares `from_a1_to_rc` output before and after on
      the formulas of the vendored suites (fixtures, ironcalc-calc), or a property test over the parser's test
      corpus.
- [x] The argument-coverage census in the eval report is unchanged (main's and this branch's `rustsheet-eval run`
      reports compared, see Outcome).
- [x] `cargo bench -p rustsheet --bench import` before and after in the Outcome; the calc benches do not slow by more
      than 2% (the change is import-only).
- [x] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure (corpus-bench fetched and
      checked; calipers and poi not fetched in this container); `make check` green; reviewers approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's snapshot and volatile scan (untimed).

## Outcome
`from_a1_to_rc` keeps the R1C1 text of a sheet's last eight formulas by parsed tree (`RcTextCache` in
crates/rustsheet-xlsx/src/import/worksheets.rs, one per `read_sheet_data`). A1 parsing stores relative references as
offsets from the cell, so a copied formula parses to an equal tree, and the R1C1 text is a function of the tree and of
the array-formula flag alone: a hit skips the legacy-`@` pass and the stringify (and the tree clone they need). Every
formula is still parsed, so the nesting-cap refusal and every parse result are as before.

Numbers (release build, this container):
- Callgrind of loading corpus-bench/52450 (`Workbook::from_xlsx_bytes`, recalc off): 12.2G instructions -> 7.9G;
  `from_a1_to_rc` 10.7G -> 6.2G (of which `Parser::parse` 5.7G, unchanged).
- `rustsheet-eval run --suite corpus-bench --filter 52450`: about 2,450-2,730 ms -> about 2,000 ms; the case thread's
  split after the change: load about 1,100 ms (from 1,550), recalc 30 ms, argument-coverage census about 770 ms (734
  ms of it the census's own A1 parse of 31,729 cells; parser construction is negligible).
- `cargo bench -p rustsheet --bench import`: median about 600 ms on main -> about 395 ms (min 568 -> 388 ms).
  The calc benches do not touch the importer.

The 1,500 ms criterion is not met here: what is left is the two A1 parses per cell (import and census), about 1.3 s
together. Skipping them needs a key cheaper than the parse that decides the tree; the audit of where the A1 parse
reads its cell (range corner order for mixed `$`, identifiers read as coordinates, structured references, LET/LAMBDA
name text, parse errors) is in T-3326, which takes the target over.

Review found that the derived `PartialEq` holds `-0` equal to `0` while the printers keep the sign (`{-0}` and `{0}`
in neighbouring cells would share text): trees holding a negative zero (`Node::has_negative_zero`, a number or an
array constant element) are neither looked up nor cached; `rc_text_cache_keeps_the_sign_of_zero` checks both orders.
`f64` fields are the only ones in `Node` and `ArrayNode` that compare equal while printing differently.

Census and eval: `rustsheet-eval check --suite corpus-bench`: known failures 3 -> 3, 0 regressions, 0 fixes; the
`argument_coverage` section and every case's census are identical in main's and this branch's corpus-bench reports
(912 cases); `make check` (gate suites) green.

Tests: `rc_text_cache_prints_what_a_fresh_parse_prints` (worksheets/tests.rs) converts every formula of the vendored
workbooks (crates/rustsheet-xlsx/tests/calc_tests and eval/suites/fixtures), as written and copied one row down and
one column right (copies hit the cache), plain and as array formulas, through one shared cache and through a fresh
one, and requires equal results; it fails if the cache ignores the array flag.
