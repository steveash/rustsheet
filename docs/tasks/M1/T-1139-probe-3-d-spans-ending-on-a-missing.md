---
id: T-1139
title: "Probe 3-D spans ending on a missing sheet, @ of a 3-D span, and IF-wrapped spans in AREAS, VLOOKUP, HLOOKUP, COUNTBLANK"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence replaces the three engine choices about 3-D sheet spans that docs/excel-semantics/references.md marks UNVERIFIED: a span whose end is not a sheet, explicit implicit intersection of a span, and IF-wrapped spans inside AREAS, VLOOKUP/HLOOKUP and COUNTBLANK.

## Context
M1 scope "References: cross-sheet". references.md, 3-D span entry (Tasks T-0122, T-0197, T-1026): "Status: VERIFIED, except two engine choices without a probe (UNVERIFIED): a span whose end is not a sheet (`=SUM(Data1:Nope!A1)`) is `#REF!` like any reference to a missing sheet, and the explicit implicit intersection `=@Data1:Data3!A1` is `#VALUE!`", and "IF-wrapped spans inside AREAS, VLOOKUP/HLOOKUP and COUNTBLANK share that path and are UNVERIFIED (no probe)". T-1015 probed INDEX/MATCH/SUM over `IF(TRUE,span)` (probes/3d-ref-functions C15:C22) but not these functions. Code: `get_sheet_span_value` / `get_sheet_span_reference` (crates/rustsheet-engine/src/reference_operators.rs), `evaluate_single_sheet_arg` with `IfBranchMode::SingleSheet` (functions/util.rs).

A formula naming a missing sheet may be stored by Excel as `#REF!` or may make Excel for the web refuse the workbook. Put each missing-sheet form in its own scratch case first; commit the ones that load and record a refusal (message, date) in references.md.

Follow the `excel-probe` skill, on the Data1/Data2/Data3 layout of probes/3d-ref-functions. New case probes/3d-ref-span-edges with at least:
- `=SUM(Data1:Nope!A1)`, `=SUM(Nope:Data3!A1)`, `=ISREF(Data1:Nope!A1)`.
- `=@Data1:Data3!A1`, `=@Data1:Data1!A1`, `=SUM(@Data1:Data3!A1:A2)` and FORMULATEXT of the first.
- `=AREAS(IF(TRUE,Data1:Data3!A1:A2))`, `=AREAS(IF(TRUE,Data1:Data1!A1:A2))`, `=VLOOKUP(1,IF(TRUE,Data1:Data3!A1:B2),2,FALSE)`, `=HLOOKUP(1,IF(TRUE,Data1:Data3!A1:B2),2,FALSE)`, `=COUNTBLANK(IF(TRUE,Data1:Data3!A1:B3))`, `=COUNTBLANK(IF(TRUE,Data1:Data1!A1:B3))`.

## Acceptance criteria
- [ ] probes/3d-ref-span-edges is committed with an excel-graph oracle sidecar for every form that loads; refused forms are listed in references.md with message and date.
- [ ] references.md's 3-D span entry cites the new cells for each of the three former UNVERIFIED points and no longer marks them UNVERIFIED (each becomes VERIFIED (web) or a recorded load refusal).
- [ ] Engine mismatches are recorded with `make eval-baseline` as new-coverage failures only, and a follow-up engine task owning them is filed and named in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (file the follow-up instead).
- Sheet insert/delete/move edits of spans (references.md "3-D references when sheets move or are renamed", T-0198).

## Outcome
Added probes/3d-ref-span-edges with an excel-graph sidecar (13 cells); every form loaded. Excel: a span ending on a missing sheet is `#NAME?`, `@` of a multi-sheet span is `#REF!`, IF-wrapped spans in AREAS/VLOOKUP/HLOOKUP/COUNTBLANK are `#VALUE!`. references.md updated (VERIFIED (web)). Eval delta: probes known failures 364 -> 370 (6 new-coverage, 0 regressions); follow-up T-1170 owns them.
