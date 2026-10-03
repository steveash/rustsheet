---
id: T-0918
title: "Port IronCalc INDIRECT (R1C1) fixes and triage remaining 2026-10-01..02 upstream commits"
milestone: M9
lane: functions
status: done
priority: P3
size: M
depends_on: [T-0911]
---

## Goal
Continue T-0911: upstream INDIRECT fixes `b65478c8` (R1C1) and `72afa4a0` (INDIRECT.xlsx, adds
`utils.rs` ref parsing), then triage POWER ranges `daf5f6c6`, text functions on arrays `35934443`,
finite-number parse `58279cda`/`80eafbbc`, `--A1:B4` `9b1ae3fd`, implicit-intersection export
`9212095d`.

## Context
Patches apply with `git format-patch <sha> --stdout --binary`, path-rewriting `base/src` to
`crates/rustsheet-engine/src` and `xlsx/tests` to `crates/rustsheet-xlsx/tests`, then `git am -3`.
Upstream INDIRECT static-analysis test lines were dropped from T-0911's port (not yet applicable).

## Acceptance criteria
- [ ] Each upstream commit ported (with `Ported-From: ironcalc@<sha>`) or a note on why not.
- [ ] Excel-authored upstream workbooks cited as evidence; `make check` green.

## Outcome
Ported `b65478c8` (INDIRECT R1C1) and `72afa4a0` (INDIRECT.xlsx + `utils.rs` ref parsing; 114 new
checked cells in ironcalc-calc, all passing) and `35934443` (text functions on arrays). The other
five commits do not apply cleanly to the fork; filed as T-0920.
