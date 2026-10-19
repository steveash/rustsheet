---
id: T-1936
title: "Move LEGACY_SPELLINGS to a spelling Excel refuses as a defined name"
milestone: M2
lane: engine
status: done
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
- [x] A unit test with a LAMBDA name of the old spelling `_RS_LEGACY_N` shows `=_RS_LEGACY_N(1)` calls the name (17-style result), and legacy N/T round-trips through R1C1 text and old `.ic` files still reload.
- [x] The chosen spelling has its own oracle-confirmed refusal evidence before it ships (lowercase `r`/`c`, single `C` and a 255-character name were not probed in T-1912; a 255-character name is accepted).
- [x] probes/legacy-marker-spelling-name-plain Sheet1!A1 and A2 (`=_RS_LEGACY_N(1)`, `=_RS_LEGACY_T(1)`, Excel 17) pass; the baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The marker is stored as `NLG1(`/`TLG1(` (`LEGACY_SPELLINGS`, functions/kinds.rs), read in R1C1
(stored) text only. The probes found no name Excel refuses that the stored-text lexer reads as a
call name (`R`, `RC`, `R1C1` are R1C1 references there), so the spelling is a cell instead: Excel
opens a workbook defining a LAMBDA name `NLG1`/`TLG1`, but `NLG1(1)` is `#REF!` and `ISREF(NLG1)`
TRUE (probes/legacy-marker-spelling-name-nlg1, -tlg1), so no callable name can have it. In A1
text it is a cell call, `#REF!`, as in Excel. `=_RS_LEGACY_N(1)` now calls a LAMBDA name of that
spelling, or is `#NAME?` without one, also after a reload (probes/legacy-marker-spelling-undefined).
T-1851's stored spelling is no longer read as the marker: stored text cannot tell it from a call of a
name of that spelling (fidelity review), and the model file has no version to key on, so a marked
cell in a file saved in the one day between T-1851 and T-1936 reloads as that name call (recorded in
references.md; such files still load). Eval delta:
probes fixed 2 (legacy-marker-spelling-name-plain A1:A2), new case legacy-marker-spelling-undefined passes, new-coverage failures 2 (the two new
cases are `!EngineError`: ours refuses the cell-like defined name, follow-up T-1997), regressions 0.

