---
id: T-2249
title: "Generator edge pools for M9 engineering and database params: complex numbers, base-n strings, Bessel orders, units and criteria ranges"
milestone: M9
lane: harness
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063]
---

## Goal
M9 engineering and database params draw from edge pools. The decision 0094 sweeps then hit the inputs where Excel's behavior changes, not just
random interior values.

## Context
- T-2109 and T-2110 added pools for M2–M6 (`number_edges`, `unit_interval`, `angle`,
  `code_point`, `text_edges`, `numeric_text`, `number_formats`, `errors`). The M7 and M9 params
  have none (0 pooled params among 263 M9 and 32 M7 functions).
- Edges to cover:
  - complex strings: "i", "-i", "3+0i", "0", "1e308+1e308i", the `j` suffix, malformed strings;
  - base-n strings: 10-character two's-complement negatives, "0", the maximum, invalid digits,
    leading zeros;
  - `places`: below the length, 10, 11;
  - Bessel x and n: 0, near the 8 cut, 1E3, negative, fractional n, n = 100000 and above;
  - CONVERT unit pairs, including prefixed and invalid units;
  - D* criteria ranges: wildcards, comparison operators, blank criteria, several criteria rows.
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
Sixteen pools in tools/src/rustsheet_tools/gen/pools.py are set on all 118 params of the 66 M9 engineering and
database functions (eval/catalog/functions.yaml); no param of them is left without a pool:
- complex_text and complex_suffix: IM* inumbers and COMPLEX's suffix.
- bin_digits, oct_digits, hex_digits, dec_number and places: the base conversions.
- bessel_x and bessel_order.
- convert_unit.
- erf_x.
- bit_number and bit_shift.
- db_table, db_field and db_criteria.
COMPLEX, CONVERT's number, DELTA and GESTEP take the existing number_edges.

The D* pools point into a fixed sheet, `Db` (new tools/src/rustsheet_tools/gen/database.py): a table and fourteen
criteria blocks. A case gets this sheet only when one of its formulas refers to it, so other cases do not change. A
new catalog key `pool_rate` (0.8 on the D* database and criteria) makes most D* calls pair a Db table with Db
criteria. Each pool's comment gives the Excel reason for its edges.

Smoke check: a scratch suite of 790 cells, filled through excel-graph and then deleted, had a value in every cell.
Ours matched 760 of them. The new quirks are in engineering.md and criteria.md ("edge-pool smoke check (T-2249)")
and are filed as T-2420 (IM* leading +, OCT2DEC of 11 digits, OCT/HEX of TRUE), T-2421 (BESSELK underflow,
BITLSHIFT ±53) and T-2422 (D* criteria: unknown header, computed criteria, header-only range, trailing blank
records). Excel's BESSELI above the order cap (0) is added to the DEVIATION entry.
Eval delta: none (the sweep task T-2258 regenerates the suites).
