---
id: T-1582
title: "Probe lone surrogates through CODE, comparisons, case and search functions (SPEC-0010 step 8)"
milestone: M2
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how Excel treats lone surrogates in the functions SPEC-0010 leaves on the lossy view, recorded in docs/excel-semantics with follow-up tasks.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md "Unprobed". Build lone surrogates with MID (e.g. `MID("😀",1,1)`, `MID("😀",2,1)`) and probe: CODE of each; `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")` (does `&` re-pair?); `=`, `<`, EXACT, MATCH and SORT with lone surrogates and with U+FFFD (e.g. `EXACT(MID("😀",1,1),UNICHAR(65533))`: after T-1579 the lossy view makes it TRUE, so a FALSE from Excel is a follow-up); UPPER/LOWER/PROPER/TRIM/CLEAN; SUBSTITUTE/FIND/SEARCH with a lone half as find_text (can they split a pair?); VALUE; CONCAT and TEXTJOIN truncation at 32,767 inside a pair; how Excel writes a lone *high* surrogate in a cached value (expected `_xD83D_`; only a lone low has been observed). Use the excel-probe skill; return only LEN/UNICODE/booleans where a cell value would be a lone surrogate, so the snapshot is unambiguous.

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with oracle values (excel-graph).
- [ ] Findings in docs/excel-semantics/text.md (and coercion.md for truncation), each with evidence cells.
- [ ] Follow-up implementation tasks filed with `make new-task` for each divergence found.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.

## Outcome
Added probes/text-lone-surrogates (87 cells, excel-graph values). Findings in docs/excel-semantics/text.md; the CONCAT/TEXTJOIN limit rows already agreed with the existing `#CALC!` entry. Divergences filed as T-1585, T-1586, T-1587. Eval: new case, 40/87 cells pass, 47 recorded as known failures.
