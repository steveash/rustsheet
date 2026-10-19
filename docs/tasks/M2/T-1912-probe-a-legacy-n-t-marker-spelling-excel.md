---
id: T-1912
title: "Probe a legacy N/T marker spelling Excel refuses as a defined name"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Find out with the oracle whether Excel accepts `_RS_LEGACY_N` and `_RS_LEGACY_T` as defined (LAMBDA) names, and find a call spelling that Excel refuses as a defined name and the engine's lexer reads as one call name.

## Context
T-1851 stores a legacy N/T as the reserved call `_RS_LEGACY_N(` / `_RS_LEGACY_T(` in R1C1 text (`functions/kinds.rs`, SPEC-0012 "Storage"). A typed or imported call of that spelling is plain N/T, so a workbook LAMBDA name of that spelling is shadowed. docs/excel-semantics/references.md "Legacy N and T marker spelling" records this as UNVERIFIED (probe pending), since decision 0052 allows no DEVIATION here. Candidates: a name Excel rejects outright, such as one longer than 255 characters (cost: stored length), or `_xlfn.`-prefixed or R1C1-shaped spellings, if Excel refuses those as names. A model file saved before T-1851 that called such a LAMBDA name reloads it as the marker, so the fix also covers old `.ic` files. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case records whether Excel loads a workbook defining `_RS_LEGACY_N` as a LAMBDA name, and what `=_RS_LEGACY_N(1)` gives there.
- [ ] Either a refused spelling is found and a follow-up engine task moves `LEGACY_SPELLINGS` to it, or the references.md entry is updated with the evidence and names the remaining gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel accepts `_RS_LEGACY_N`/`_RS_LEGACY_T` (and the `_xlfn.` form) as LAMBDA names and calls them, so the shadowing gap is real. Refused spellings found: `R`, `RC`, `R1C1`, and names over 255 characters (workbook refuses to open). Two oracle-filled cases added (`legacy-marker-spelling-name-plain`, `-xlfn`); references.md updated; follow-up T-1936 (engine, opus) moves `LEGACY_SPELLINGS`. Eval delta: 2 new probe cases, both filled, no scored regressions expected.
