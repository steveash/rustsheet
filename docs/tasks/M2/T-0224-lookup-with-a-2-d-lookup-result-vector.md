---
id: T-0224
title: "LOOKUP with a 2-D lookup/result vector: probe Excel's error (engine returns #ERROR!)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
LOOKUP with a 2-D lookup_vector or result_vector returns the error Excel returns (probe it); today it returns the non-Excel `#ERROR!`.

## Context
T-0210 made MATCH return `#N/A` for 2-D arrays and left LOOKUP unchanged (`as_vector(.., Error::ERROR)` in `lookup_and_reference/mod.rs`) because it is unprobed. Use the excel-probe skill, then record the rule in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [x] Also probe `MATCH(2,{1,2;3,4},0)` and `MATCH(2,A1:B2*1,0)`; mark the lookup.md array sub-rule VERIFIED or fix it.
- [x] Probe evidence cited; LOOKUP on a 2-D vector matches Excel; unit test; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed LOOKUP's vector form and MATCH on 2-D arrays with two new excel-graph cases, 32 cells.
Excel does not give one error for 2-D vectors. Instead (lookup.md "LOOKUP vector form with 2-D or
short vectors"):
- A 2-D lookup_vector is searched as the array form searches, along its first column, or its first
  row when wide.
- A 2-D result_vector is `#N/A`.
- A result range shorter than the match position reads on past its end (down for a column, right
  for a row or a single cell). A short result array is `#N/A`.
- MATCH on a 2-D array is `#N/A`, so the lookup.md sub-rule is now VERIFIED.

`fn_lookup` now shares one table path between its two forms (`lookup_table`), and `lookup_result`
reads the result vector; `SearchVector::into_value` became dead and was removed. The result range
is still evaluated in full, so a result range holding the formula's own cell stays circular. The
engine no longer returns `#ERROR!` from LOOKUP.
Eval: probes +32 checked cells, all passing (new coverage); 0 regressions.
