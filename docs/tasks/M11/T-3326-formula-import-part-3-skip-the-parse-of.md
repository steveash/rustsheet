---
id: T-3326
title: "Formula import, part 3: skip the parse of copied formulas (relocation key) to reach 1,500 ms on corpus-bench 52450"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3318]
---

## Goal
corpus-bench/52450 (36,640 formula cells, copies of a few COUNTIFS formulas) runs in at most 1,500 ms in
`rustsheet-eval run --suite corpus-bench --filter 52450` on a release build, with every imported formula's R1C1 text
and the argument-coverage census unchanged. T-3318 took the case from about 2,500 ms to about 2,000 ms; what is left
is two A1 parses per formula cell: the importer's (`from_a1_to_rc`, about 5.7G instructions of the load's 7.9G) and
the harness's argument-coverage census (`call_arguments`, about 730 ms of the case's time, 95% of it the parse).

## Context
T-3318 caches the R1C1 text by parsed tree (`RcTextCache` in crates/rustsheet-xlsx/src/import/worksheets.rs), which
removes the stringify and the legacy-`@` pass for copies but still parses every copy. Skipping the parse needs a key
cheaper than the parse that decides its tree: a lexer pass whose key is the formula text with each reference token
replaced by its offsets from the formula's cell. T-3318's audit of where the A1 parse reads the cell it is in (the
key must cover each, or the formula must not be cached):
- `reference_operand` and `range_operand` (parser/operands.rs) subtract the cell from relative coordinates; a range
  token first orders its corners by their absolute values, so for a corner pair of mixed `$` the order depends on the
  cell (the key needs the two comparison results);
- identifiers read as coordinates: dotted ranges (`dotted_range_operand`) and a callee spelled like a cell
  (`A1(...)`, the `parse_reference_a1` branch of the call path in operands.rs) become references relative to the cell;
- structured references read the table extent at the cell (`structured_reference.rs`, this-row forms);
- LET and LAMBDA name binding read the raw text of the argument (`source_since`, `bind_bare_name`: `LET(x1,..)`
  binds the text `x1`, so two copies with equal offsets bind different names);
- parse errors carry the formula text and positions;
- the context sheet (defined-name scope, sheet spans).
A whitelist key (references, literals, operators, punctuation, and identifiers that name a known function other than
LET and LAMBDA; anything else is not cached) avoids most of these. The census (`observe` in
crates/rustsheet-eval/src/argument_coverage.rs) can then cache the A1 tree by the same key and resolve relative
references per cell, since its shapes are a function of the tree and the cell.

## Acceptance criteria
- [x] corpus-bench/52450 takes at most 1,500 ms as above; the Outcome gives callgrind's `from_a1_to_rc` count before
      and after and the case's time split (load, census).
- [x] The key's soundness is argued in its doc comment against the list above, and a test shows equal keys giving
      equal trees on the vendored workbooks' formulas copied across cells (extending T-3318's
      `rc_text_cache_prints_what_a_fresh_parse_prints`), including mixed-`$` ranges, LET names and dotted ranges.
- [x] The argument-coverage census in the eval report is unchanged on a full `make eval`.
- [x] `cargo bench -p rustsheet --bench import` before and after in the Outcome.
- [x] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure; `make check` green;
      reviewers approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's snapshot and volatile scan (untimed).


## Outcome
Copies of a stored formula are no longer parsed on import: `import/relocation_key.rs` lexes each
formula into a relocation key (references as offsets from the cell, or values when `$`, plus a
range's corner comparisons; only called stored functions other than LET/LAMBDA as identifiers) and
`RcTextCache` returns the R1C1 text of an earlier copy with the same key. Soundness is argued in the
module docs against T-3318's audit list; `relocation_key_gives_equal_trees_for_equal_keys` and the
extended T-3318 vendored-workbook copy test check it. The census cache in `observe` was not needed
for the target and is not done.
- corpus-bench/52450 (`rustsheet-eval run --suite corpus-bench --filter 52450`, release, case ms
  from the report): main 1,710 / 1,607 / 1,673 ms, branch 1,239 / 1,348 / 1,218 ms.
- Callgrind of that run (release binaries of main eeda7cc and this branch): `from_a1_to_rc`
  6.73G -> 4.13G instructions (now almost all `KeyLexer::key`, 4.05G; the parse it skips is gone),
  `load_from_xlsx_bytes_with_limits` 8.23G -> 5.61G, the case thread 16.18G -> 13.51G. Split of the
  branch's case thread: load 5.61G (about 42%, about 530 ms), the argument-coverage census's
  re-parse most of the remaining `Parser::parse` 6.41G (about 47%, about 600 ms), calculation the rest.
- `cargo bench -p rustsheet --bench import`: min 380.2 -> 272.3 ms, median 386.6 -> 295.6 ms.
- Full eval with calipers, corpus-bench and poi fetched, main's and this branch's binaries: the
  `argument_coverage` section of report.json is identical; the only case differences are the
  T.INV / BETA.DIST / typed-parse cases main changed since this branch's base (T-2724, T-2510,
  T-3283), none in an xlsx-import path. `make eval-check`: 0 regressions in every suite
  (corpus-bench 3 -> 3, poi 115 -> 115). Eval delta: none.
- `make check` green. Review hardening: a formula holding the key's marker U+0001 is not keyed.
- Follow-up T-3384: `KeyLexer::key` is now the load's hot spot (about 110k instructions
  per formula, a full re-lex plus a `peek_token` per identifier).
