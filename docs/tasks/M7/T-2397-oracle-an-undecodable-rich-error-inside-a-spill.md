---
id: T-2397
title: "Oracle: an undecodable rich error inside a spill is scored as an expected blank; leave it pending instead"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
A cell Excel fills with a rich-value error the harness cannot decode is never scored, inside a spill as well, and gen-m7's 36 such cells stop counting as known failures.

## Context
Re-split from T-2359. `rustsheet-oracle fill` drops `#UNDECODED!` cells from the sidecar (`_drop_undecoded` in
tools/src/rustsheet_tools/oracle/cli.py) so they stay pending. But the runner reads a spilled cell that has no sidecar entry as
an expected blank, so the dropped cell is scored as empty. probes/groupby-lifted-args (excel-graph, 2026-10-22, T-2359) shows
it: `GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,REGEXEXTRACT("a","[0-9]",-1)))` spills `#VALUE!` (ERROR.TYPE 3 through INDEX, K9),
but excel-graph returns the spilled cells as rich errors. Some were reported UNDECODED and some were not reported at all (B33:B36),
and every one ended up as an expected blank until the probe read them through INDEX.

Owned cells, 36 lines in eval/baseline/gen-m7.txt: gen-m7/18b80d3fd292d63a.case.json C951:E962,
`=_xlfn.GROUPBY(Data!A1:B12,Data!B1:D12,LAMBDA(w,(REGEXEXTRACT(Data!A5,"[0-9]",(-1))/-Data!D10)))`. Excel <empty>, ours `#VALUE!`;
Excel's real value is `#VALUE!` (the probe), so ours is right.

Fix: record undecoded cells explicitly in the sidecar (for example the `#UNDECODED!` marker, which the runner never scores)
instead of dropping them. Also find why some rich errors in a spill are not reported at all. Then re-fill the case.

## Acceptance criteria
- [x] A spilled undecoded rich error is pending (not scored) after a fill; a tools or harness test covers it.
- [x] The 36 lines leave eval/baseline/gen-m7.txt with 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Decoding the rich error itself (excel-graph gives no error type).

## Outcome
`rustsheet-oracle fill` now keeps undecodable rich errors in the sidecar as `#UNDECODED!` (`_report_undecoded`)
instead of deleting them. The runner already skips the marker for scoring, and it is a known cell for the spill-extent
check, so a spill member is no longer scored as an expected blank. `UNDECODED` moved to `rustsheet_tools.snapshot`
so fitlab sweeps can ignore it too. Override evidence (`overrides.rs`) and the fill's hand-disagreement notes also
ignore it. Tests: `undecoded_spill_members_are_not_expected_blanks` (spill_extent.rs), the overrides evidence test,
and tools tests for the fill and fitlab `select`.
gen-m7/18b80d3fd292d63a was re-filled from excel-graph. It gained 38 `#UNDECODED!` cells (C951:E962 plus the pending
anchors A1036 and A1519), and no existing value changed.
Missing cells: a fresh excel-graph recalc of the probe's original `GROUPBY(...,REGEXEXTRACT("a","[0-9]",-1))`
reported all four spilled value cells (stored as `vm` to `errorType, ptg, subType` = 2, 6, 0). The earlier
"not reported" cells did not reproduce, so they are likely an artifact of the earlier probe layout.
This is recorded in docs/excel-semantics/arrays.md.
Eval delta: gen-m7 known failures -36, 0 regressions.
Follow-ups: T-2417 (decode errorType 2), T-2418 (re-fill other sidecars that dropped undecoded cells).
