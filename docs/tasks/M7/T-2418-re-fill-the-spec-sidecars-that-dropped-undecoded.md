---
id: T-2418
title: "Re-fill the spec sidecars that dropped undecoded rich errors so their spill members stop scoring as expected blanks"
milestone: M7
lane: harness
status: done
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

## Outcome
Found by scanning every gen-/probes case with baseline failures for cells where Excel is `<empty>`, ours is an
error (`rustsheet-eval show`): 13 cases (12 gen-m7 + probes/t2414-gen-m7-unexplained). Re-filled all with
`rustsheet-oracle fill -b excel-graph --force`. Result: 10 sidecars gained cells (all `#VALUE!` spill members that the
old fill had dropped; 21b4eef1 gained none), `eval/baseline` lost 1 failure (gen-m7/7307132b Calc!A587) and gained 0;
checked-cell counts rose for 10 cases.
Not committed, because they changed more than added cells: gen-m7/0624b684889e9228 (the re-fill replaced 40+
`#UNDECODED!` cells with decoded `#VALUE!`, i.e. T-2417 territory) and probes/t2414-gen-m7-unexplained (input hash
changed); gen-m7/7ea6ef6ceabacafd only differed in `source` fields, also left out. Follow-up: re-fill 0624b684 when T-2417 lands.
