---
id: T-3318
title: "Formula import, part 2: corpus-bench 52450 to 1,500 ms (stringify and legacy-@ pass, argument-coverage re-parse)"
milestone: M11
lane: perf
status: todo
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
- [ ] corpus-bench/52450 takes at most 1,500 ms as above; the Outcome gives callgrind's `from_a1_to_rc` count before
      and after (target at most 7G) and the case thread's.
- [ ] Every imported formula's R1C1 text is unchanged: a test compares `from_a1_to_rc` output before and after on
      the formulas of the vendored suites (fixtures, ironcalc-calc), or a property test over the parser's test
      corpus.
- [ ] The argument-coverage census in the eval report is unchanged on a full `make eval`.
- [ ] `cargo bench -p rustsheet --bench import` before and after in the Outcome; the calc benches do not slow by more
      than 2%.
- [ ] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure; `make check` green;
      reviewers approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's snapshot and volatile scan (untimed).
