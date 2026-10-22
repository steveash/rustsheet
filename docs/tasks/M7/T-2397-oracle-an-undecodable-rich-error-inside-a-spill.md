---
id: T-2397
title: "Oracle: an undecodable rich error inside a spill is scored as an expected blank; leave it pending instead"
milestone: M7
lane: harness
status: in-progress
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
- [ ] A spilled undecoded rich error is pending (not scored) after a fill; a tools or harness test covers it.
- [ ] The 36 lines leave eval/baseline/gen-m7.txt with 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decoding the rich error itself (excel-graph gives no error type).
