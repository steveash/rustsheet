---
id: T-2418
title: "Re-fill the spec sidecars that dropped undecoded rich errors so their spill members stop scoring as expected blanks"
milestone: M7
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every spec-suite sidecar written before T-2397 that dropped an undecoded rich error is re-filled, so its spill members hold `#UNDECODED!` (pending) instead of being missing (scored as an expected blank).

## Context
Before T-2397, `rustsheet-oracle fill` deleted `#UNDECODED!` cells from sidecars. The runner scores a spill member
that has no sidecar entry as an expected blank, so a dropped member that our engine fills shows up as a known failure
with Excel `<empty>` (gen-m7/18b80d3fd292d63a C951:E962 was one, fixed in T-2397). To find the others, look at
baseline failures where Excel is `<empty>`, ours is an error, and the cell lies inside the spill of a formula anchor in
the same case (`make eval-show CASE=...`), especially GROUPBY/PIVOTBY/MAP/BYROW cases in gen-m7 and probes. Re-fill
each one with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite <suite> --case <suite>/<file>.case.json --force`
(the case id includes `.case.json`). Then check that the fill's UNDECODED lines name the cells, run `make eval-baseline` and commit.

## Acceptance criteria
- [ ] The candidate cases and how they were found are listed in the Outcome.
- [ ] Each re-filled sidecar differs from the old one only by added `#UNDECODED!` cells and the `source` fields (any other change is reported, not committed blindly).
- [ ] The cells leave eval/baseline with 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decoding the errors (T-2417).
