---
id: T-0911
title: "Port IronCalc INDIRECT (R1C1) and ROW/COLUMN-on-range fixes"
milestone: M9
lane: functions
status: todo
priority: P3
size: M
depends_on: []
---

## Goal
Upstream fixed INDIRECT reading R1C1 references (`b65478c8`, `72afa4a0`) and ROW/COLUMN
accepting ranges (`b6d34abc`). Evaluate each against Excel and port what matches.

## Acceptance criteria
- [ ] Excel evidence cited for each ported behavior (`excel-probe` skill if none exists).
- [ ] Spec cases added; behavior matches; `make check` green.
- [ ] Commits carry `Ported-From: ironcalc@<sha>` for each upstream change used.
- [ ] Remaining upstream commits in the 2026-10-01..02 window (POWER ranges `daf5f6c6`, text
  functions on arrays `35934443`, finite-number parse `58279cda`/`80eafbbc`, `--A1:B4`
  `9b1ae3fd`, implicit-intersection export `9212095d`) are triaged: port or note why not.
