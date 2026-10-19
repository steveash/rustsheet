---
id: T-1936
title: "Move LEGACY_SPELLINGS to a spelling Excel refuses as a defined name"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A workbook LAMBDA name is never shadowed by the stored spelling of a legacy N/T: the engine stores the marker under a spelling Excel refuses as a defined name.

## Context
T-1912 found (excel-graph, 2026-10-18) that Excel accepts `_RS_LEGACY_N`, `_RS_LEGACY_T` and `_xlfn._RS_LEGACY_N` as defined LAMBDA names and calls them (probes/legacy-marker-spelling-name-plain, -xlfn: 17, where N(1) is 1). So `LEGACY_SPELLINGS` (`functions/kinds.rs`) shadows such a name today. Excel refuses these spellings as names (the workbook then fails to open, createSession 501 `unsupportedWorkbook`): `R`, `RC`, `R1C1`, and any name over 255 characters (a 256-character name was refused; 255 was accepted). Candidates must also be read by the engine's lexer as one call name in R1C1 text (check `R1C1(` is not lexed as a reference) and cost little stored length. See references.md "Legacy N and T marker spelling".

## Acceptance criteria
- [ ] A unit test with a LAMBDA name of the old spelling `_RS_LEGACY_N` shows `=_RS_LEGACY_N(1)` calls the name (17-style result), and legacy N/T round-trips through R1C1 text and old `.ic` files still reload.
- [ ] The chosen spelling has its own oracle-confirmed refusal evidence before it ships (lowercase `r`/`c`, single `C` and a 255-character name were not probed in T-1912; a 255-character name is accepted).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
