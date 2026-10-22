---
id: T-2247
title: "Generator edge pools for M9 statistical params: probabilities, degrees of freedom, shapes, tails and data arrays"
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
M9 statistical params draw from edge pools. The decision 0094 sweeps then hit the inputs where Excel's behavior changes, not just
random interior values.

## Context
- T-2109 and T-2110 added pools for M2–M6 (`number_edges`, `unit_interval`, `angle`,
  `code_point`, `text_edges`, `numeric_text`, `number_formats`, `errors`). The M7 and M9 params
  have none (0 pooled params among 263 M9 and 32 M7 functions).
- Edges to cover:
  - probabilities: 0, 1, 1E-300, 1E-16, 0.5, 1-1E-16, just inside and outside (0,1);
  - degrees of freedom and shapes: 1, 2, half-integers, 1E-10, 1E5, 1E10, non-integers that
    truncate;
  - x near 0, the mode, the median and the deep tails;
  - booleans for `cumulative`, as TRUE/FALSE, 0/1 and text;
  - data arrays: one value, constant values, blanks/text/booleans mixed in, and large magnitudes
    that cancel.
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
Added six pools to `gen/pools.py`, each edge commented with its statistics.md reason: `probability` (0, 1, 1E-300 to
1E-10, the test levels, 1-1E-16 and 1-1E-17, the first values outside [0, 1]), `shape` (1, 2, values that truncate,
below 1, 0, negatives, 1E-300 to 1E-10, 1E5, the 1E10 df cut and just past it), `stat_x` (0 and +-1E-300, the mode and
median region, the normal's deep tails at 38-40, 1E10 to 1E307), `gamma_x` (poles, between poles, 25/26, the 171.6/171.7
overflow edge, 1E300), `flag` (TRUE/FALSE, 0/1/2/-1/0.5, logical and other text, an error) and `stat_data` (a single
value as scalar and array, constant and two-value data, text/booleans/blanks inside, an error element, a big common
offset, cancelling and overflowing magnitudes, ties, a row). `pool:` is set on 188 M9 statistical params: probability,
probability_s, the alphas of BINOM.INV/CRITBINOM/CONFIDENCE*, PERCENTILE's k and TRIMMEAN's percent; every df, standard
deviation, sigma, lambda, the BETA/GAMMA/WEIBULL shapes and POISSON's mean; the distributions' x and z; GAMMA/GAMMALN x;
every cumulative and LINEST/LOGEST/GROWTH/TREND const and stats; the data of the aggregates, order statistics, paired and
x/y regression functions. FISHER's x went on `unit_interval`, which already named it. No generator change: `array` and
`aggregate` params already reach the pool through `scalar_text`. Smoke check: a scratch probe case with one formula per
pool entry (123 cells) filled through excel-graph returned a value for all 123; the case was deleted. No eval change.
