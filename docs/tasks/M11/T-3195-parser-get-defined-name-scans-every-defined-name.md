---
id: T-3195
title: "Parser::get_defined_name scans every defined name with to_lowercase on each lookup: name-heavy calipers workbooks spend 68-96% of their load in it (pv23_062, pv23_002, pv26_018)"
milestone: M11
lane: perf
status: todo
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
- [ ] `Parser::get_defined_name` does no per-entry `to_lowercase` and no scan of the defined-name list. A unit test covers
      a sheet-scoped name shadowing a workbook name, mixed case, a non-ASCII name (`ß`, `İ`) and two names folding to the
      same key.
- [ ] calipers pv23_062 and pv23_002 each take at most 200 ms in `rustsheet-eval run --suite calipers --filter pv23_0`
      on a release build (from 1,450 and 1,393 ms), and pv26_018 at most 1,200 ms (from 3,272 ms). The Outcome lists the
      before/after times of the other many-name cases above.
- [ ] `make eval` with calipers fetched shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- `Model::get_parsed_defined_name` and the evaluation-time lookup (T-0410).
- Formula parse cost per character (T-0499).
