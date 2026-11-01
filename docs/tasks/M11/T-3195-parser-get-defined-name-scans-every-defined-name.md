---
id: T-3195
title: "Parser::get_defined_name scans every defined name with to_lowercase on each lookup: name-heavy calipers workbooks spend 68-96% of their load in it (pv23_062, pv23_002, pv26_018)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Resolving a name while parsing costs one keyed lookup, not a scan of every defined name, so loading a workbook with
tens of thousands of names takes time linear in their number. Name resolution (case-insensitive, sheet scope before
workbook scope) stays exactly as it is.

## Context
Gap triage 2026-10-30. `rustsheet-eval run --suite calipers` on origin/main c59497fc: calipers/pv_goldens/pv23_062
takes 1,450 ms for 19 scored cells (`E9`, `SUM(E13:E14)` and the like), and pv23_002 1,393 ms for 221. Both hold
17,787 defined names (`________________________________PR1` with body `#REF!`, and similar). Callgrind (`valgrind
--tool=callgrind ./target/release/rustsheet-eval show calipers/pv_goldens/pv23_062/golden.xlsx`), 19.65G instructions:
- `Model::parse_defined_names` 18.9G (96%);
- under it `Parser::get_defined_name` 18.6G, of which `str::to_lowercase` 11.5G and malloc/free most of the rest.

`Parser::get_defined_name` (crates/rustsheet-engine/src/expressions/parser/mod.rs) walks `self.defined_names` twice
(sheet scope, then workbook scope). For every entry it lowercases both the name asked for and the entry's name, so
parsing n name bodies costs about n² string allocations. Formulas pay the same cost at import, because
`from_a1_to_rc` parses every cell formula with the same `Parser`. calipers pv26_018 (and pv26_2_018) has only 2,686
names, yet it takes 3,272 ms. Callgrind on it, 37.7G instructions: `import::worksheets::from_a1_to_rc` 27.1G, of which
`Parser::get_defined_name` 25.7G (68% of the run) and `to_lowercase` 17.7G. List the other many-name cases in the
Outcome: pv26_010/pv26_2_010/pv23_057/pv23_059 (44,028 or 43,054 names, 1.7-2.7 s), pv26_2_017/pv26_017 (25,163),
pv26_016 (4,219), pv26_003 (2,421) and pv26_001/pv26_2_001 (13,696).

Fix: build a map once per `Parser` (or whenever its defined-name list is set), keyed by (lowercased name, scope).
Lowercase the asked name once per lookup. Keep the first match in list order if two entries fold to the same key.
The model already keys `parsed_defined_names` by `(scope, lowercased name)`: `fn_isref` (functions/information/mod.rs)
looks names up that way. T-0410 is removing the model-side scan in `Model::get_parsed_defined_name`. Reuse that key
shape and the same folding, so the parser and the model agree on non-ASCII names. Search for an existing helper before writing one (CLAUDE.md directive 3).

## Acceptance criteria
- [x] `Parser::get_defined_name` does no per-entry `to_lowercase` and no scan of the defined-name list. A unit test covers
      a sheet-scoped name shadowing a workbook name, mixed case, a non-ASCII name (`ß`, `İ`) and two names folding to the
      same key.
- [x] calipers pv23_062 and pv23_002 each take at most 200 ms in `rustsheet-eval run --suite calipers --filter pv23_0`
      on a release build (from 1,450 and 1,393 ms), and pv26_018 at most 1,200 ms (from 3,272 ms). The Outcome lists the
      before/after times of the other many-name cases above.
- [x] `make eval` with calipers fetched shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- `Model::get_parsed_defined_name` and the evaluation-time lookup (T-0410).
- Formula parse cost per character (T-0499).

## Outcome
- **Index.** `Parser` keeps `name_index`, mapping each defined name's position by (scope, lower-cased name). This is
  the key `Model::parsed_defined_names` uses, so the two fold names alike. When several names fold to the same key,
  the first in list order wins, as the scan did.
  - The index is built in `Parser::new` and again in `set_worksheets_and_names`, the only two places the name list is
    set.
- **Lookups.** `get_defined_name` lower-cases the asked name once and probes the sheet scope, then the workbook scope.
  The workbook-scope scan for a sheet-qualified name in operands.rs now goes through the same
  `defined_name_formula`.
- **Release timings,** `rustsheet-eval run --suite calipers --filter ...`: pv23_062 69 ms and pv23_002 73 ms (from
  1,450 and 1,393 ms); pv26_018 665 ms and pv26_2_018 673 ms (from 3,272 ms).
- **Load times of the many-name cases,** standalone (`load_from_xlsx`), before → after:

  | case | before | after |
  |---|---|---|
  | pv23_062 | 1,290 ms | 53 ms |
  | pv23_002 | 1,200 ms | 56 ms |
  | pv26_018 | 2,317 ms | 216 ms |
  | pv26_2_018 | 2,125 ms | 266 ms |
  | pv26_016 | 1,556 ms | 285 ms |
  | pv26_003 | 1,464 ms | 165 ms |
  | pv26_010 | 594 ms | 559 ms |
  | pv26_2_010 | 590 ms | 556 ms |
  | pv23_057 | 576 ms | 489 ms |
  | pv23_059 | 608 ms | 540 ms |
  | pv26_017 | 215 ms | 226 ms |
  | pv26_2_017 | 278 ms | 238 ms |
  | pv26_001 | 565 ms | 507 ms |
  | pv26_2_001 | 603 ms | 501 ms |
  | pv26_002 | 3.7 s | 3.9 s |

  The cases with 13,000 to 44,000 names barely move: their load is not spent resolving names in formulas. pv26_002's
  load is formula import (T-3384).
- **Tests** (expressions/parser/tests/test_defined_name_lookup.rs):
  - every lookup is compared with the old two-pass scan, over 19 queries on 3 sheets;
  - the names cover a sheet name shadowing a workbook name, mixed case, `ß`, `İ`, the titlecase `ǅ`, and names
    folding to the same key in both scopes;
  - the index is rebuilt when the name list is set again.
- **Mutation:** keeping the last of two names that fold alike fails the comparison test.
- **Not in this task:** the evaluation-time lookups, `Model::get_parsed_defined_name` (T-0410) and the reference-context
  lookups (T-1194), stay open.

