---
id: T-1214
title: "Implement BAHTTEXT"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BAHTTEXT returns Excel's Thai-language baht text for a number.

## Context
From T-1087's probe probes/text-modes-bahttext: `BAHTTEXT(5)` and `BAHTTEXT("5")` are "ห้าบาทถ้วน" in Excel and `#NAME?` in the engine. Probe more values (satang, millions, negatives, zero, large numbers) with the excel-probe skill before implementing.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/851eb2099ae3f805 Calc!A9 `=SUBSTITUTE("#N/A",Data!$E5,BAHTTEXT(SUM(Data!E9:E10)),1)`: Excel "#N/A", engine `#NAME?`.
- gen-m7/9ff3afcacd3bd0fc Calc!A38 `=(BAHTTEXT(-Data!E4)&LET(..,1e-15))` with E4 = -0.0001: Excel "ศูนย์บาทถ้วน0.000000000000001" (`BAHTTEXT(0.0001)` is "ศูนย์บาทถ้วน", zero baht), engine `#NAME?`.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A21 `=BAHTTEXT(GAMMADIST(Data!B10,FIND("abc","TRUE"),..))`: Excel `#VALUE!`, engine `#NAME?`.

## Acceptance criteria
- [ ] probes/text-modes-bahttext passes, and a wider probe covers satang, negatives and millions.
- [ ] gen-m7/851eb2099ae3f805 A9 and gen-m7/9ff3afcacd3bd0fc A38 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/4df1b22d3ed63f58 A21 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
