---
id: T-2707
title: "Probe PROB's unprobed argument orders and trace the inner calls of three T-2468 cells"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's PROB orders for the cases T-2468 implemented by inference are settled by oracle cells, and the
three T-2468 cells that kept their known-failure lines without an owner each get one.

## Context
T-2468 (PROB reads its data before the limits and pairs cells by position) left these inferred or unowned
(reviewer findings on #1771; docs/excel-semantics/statistics.md, PROB entry, the UNVERIFIED list):
- Unprobed orders: a typed boolean as x_range against a number (`PROB(-6,1=1,0,3)`, inferred `#DIV/0!` from the
  TRANSPOSE'd boolean in gen-m9/5cebac1f02bf30f2 A372); a bare typed text (`PROB(1,"a",1)`); a blank one-cell x_range
  against an error prob_range; an x_range range holding an error cell against a direct error prob_range (ours returns
  prob_range's error); text elements inside array arguments on both sides.
- Cells still failing with no owner, each needing its inner call traced first: gen-m9/17c110a53924beb8 A330 (its GROUPBY),
  d37313406b9f6e55 A142 (its inner YIELD), 80586b859b73c87d A353 (`RANK.EQ` of a PROB whose upper limit is a TINV;
  Excel's `#N/A` may be RANK.EQ's not-found). 6e18550e9fc9c1b9 A394 and 92d4fa29e5510f1e A70 are T-2685's;
  941c409532603d32 A324 is T-2425's.
- Code: crates/rustsheet-engine/src/functions/statistical/prob.rs (`prob_data`, `direct`).

## Acceptance criteria
- [x] A probe case (excel-probe skill) covers each unprobed order above, with oracle values committed; statistics.md's
      PROB entry states each result and drops it from the UNVERIFIED list.
- [x] Each of the three unowned cells is traced to the sub-expression that diverges and has an owning task (existing
      or new), recorded in this task's Outcome.
- [x] Any engine divergence the probe finds is filed as a functions-stat task (or fixed here if size S).
- [x] `make check` green; reviewers approve.

## Out of scope
- T-2685's PV cells and T-2425's FORECAST.ETS cell.

## Outcome
- probes/t2707-prob-orders (excel-graph, 2026-10-29, 74 cells) covers each unprobed order; statistics.md's PROB entry states
  each result and drops them from its UNVERIFIED list (the lower_limit-before-counts order stays UNVERIFIED: no cell separates
  them). Typed text, numeric text and booleans are one cell with no number (`#DIV/0!` against a number, A1:A8); text and
  booleans inside array constants are dropped like a range's (A24:A34).
- Two engine divergences, fixed here in `statistical/prob.rs` (size S): a blank one-cell x_range is `#VALUE!` before
  prob_range's error value (A9:A12), and unequal cell counts are `#N/A` before an array's error element (A54, A56).
  These also fix gen-m9/775d053de498b8c7 Calc!A159 and b9a32ce9a1fd0d71 Calc!A331.
- The three unowned cells: d37313406b9f6e55 A142 and 80586b859b73c87d A353 already pass on main (no owner needed).
  17c110a53924beb8 A330 traces to its GROUPBY: `total_depth` is `INDEX(Data!A4:D4,<blank>)`, the reference INDEX returns,
  which Excel gives as one `#VALUE!` under ROWS/ISERROR/ERROR.TYPE but lifts under PROB (`#N/A`, 1 cell against 4); ours is
  the reverse. Owner: **T-3088** (functions-lookup, M7, opus), with the probe's A37:A39, A43, A44, A49, A50, B51:D51, A60,
  A64 and A65 (known failures in eval/baseline/probes.txt). arrays.md records the rule.
- Eval delta: gen-m9 known failures 3309 -> 3307 (fixed 2); probes +1 case, 13 new-coverage known failures (all T-3088's);
  0 regressions.
