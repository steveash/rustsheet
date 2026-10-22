---
id: T-2286
title: "Generator edge pools for M6 and M7 params: lookup tables, array shapes, blanks and errors in arrays, date serial edges"
milestone: M7
lane: harness
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063]
---

## Goal
M6 and M7 params draw from edge pools. The decision 0094 sweeps then hit the inputs where Excel's behavior changes, not just
random interior values.

## Context
- T-2109 and T-2110 added pools for M2–M6 (`number_edges`, `unit_interval`, `angle`,
  `code_point`, `text_edges`, `numeric_text`, `number_formats`, `errors`). The M7 and M9 params
  have none (0 pooled params among 263 M9 and 32 M7 functions).
- Edges to cover:
  - lookup tables: sorted and unsorted, duplicates, blanks, mixed types, approximate-match
    boundaries;
  - array shapes: 1×1, 1×n, n×1, ragged results;
  - arrays containing errors and blanks;
  - date serials: 0, 60 (1900-02-29), 2958465, negative, fractional times;
  - REGEX* patterns: anchors, classes, `(?i)`, empty matches;
  - LAMBDA bodies for MAP, REDUCE and SCAN.
- Source edges from docs/excel-semantics/*.md (the cuts and caps already found) and from the
  existing probes, so the sweep reproduces known quirks at scale.

## Acceptance criteria
- [x] The pools are added to `tools/src/rustsheet_tools/gen/pools.py`, and `pool:` is set on the
      catalog params listed above (`eval/catalog/functions.yaml`). Each pool's edges are listed in
      a comment, with the Excel reason for each.
- [x] Tools tests: every pool value is a valid literal under 255 characters, and the draw keeps
      each param's `domain`.
- [x] Smoke check: a scratch suite with one formula per pool value, filled through excel-graph,
      returns a value for every cell. Delete the scratch suite afterwards, as T-2110 did.
- [x] No eval change: the sweep tasks regenerate the suites.
- [x] `make check` green; reviewers approve.

## Out of scope
- Generating or filling suites (the sweep tasks).

## Visible-parity stage
Prerequisite for decision 0094's sweeps; its acceptance criteria are the whole stage.

## Outcome
Added six pools to `gen/pools.py`, each edge commented with its excel-semantics reason: `array_shape` (single values,
1x1, 1xn, nx1, 2-D, ragged stacks), `array_holes` (error elements, blank cells from the never-filled Data!Z, empty
strings), `lookup_value`, `lookup_table` (sorted, descending, unsorted, duplicate runs, mixed types, case, errors,
blanks), `regex_pattern` (anchors, empty matches, empty alternation, lookahead, inline flags, POSIX/Unicode classes,
named back-references, \X, \R) and `lambda_body` (arity 1 and 2: identity, array, range, error, ISBLANK, nested
LAMBDA, VSTACK accumulator, optional parameter). `pool:` is set on 39 catalog params (TAKE, DROP, EXPAND, CHOOSEROWS/COLS,
WRAPROWS/COLS, HSTACK, VSTACK, ROWS, COLUMNS, TRANSPOSE; TOCOL, TOROW, UNIQUE, SORT, MAP, BYROW, BYCOL, REDUCE, SCAN and the
four WORKDAY/NETWORKDAYS holidays; XLOOKUP/XMATCH/LOOKUP lookup_value; XMATCH/LOOKUP lookup tables; the three REGEX
patterns; six LAMBDA params). Generator: a pooled `range` param takes a pool entry 35% of the time, and a pooled
`lambda` param an entry of the needed arity with renamed parameters. The date-serial edges were already pooled
(T-2179). Smoke check: a scratch probe case with one formula per pool entry (102 cells) filled through excel-graph
returned a value for 101; the invalid pattern `"["` came back as an undecodable rich error, so it was dropped from
`regex_pattern`. The scratch case was deleted. No eval change.
