---
id: T-3326
title: "Formula import, part 3: skip the parse of copied formulas (relocation key) to reach 1,500 ms on corpus-bench 52450"
milestone: M11
lane: perf
status: todo
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
- [ ] corpus-bench/52450 takes at most 1,500 ms as above; the Outcome gives callgrind's `from_a1_to_rc` count before
      and after and the case's time split (load, census).
- [ ] The key's soundness is argued in its doc comment against the list above, and a test shows equal keys giving
      equal trees on the vendored workbooks' formulas copied across cells (extending T-3318's
      `rc_text_cache_prints_what_a_fresh_parse_prints`), including mixed-`$` ranges, LET names and dotted ranges.
- [ ] The argument-coverage census in the eval report is unchanged on a full `make eval`.
- [ ] `cargo bench -p rustsheet --bench import` before and after in the Outcome.
- [ ] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure; `make check` green;
      reviewers approve.

## Out of scope
- Defined-name parsing on load (T-3195); calculation time; the harness's snapshot and volatile scan (untimed).
