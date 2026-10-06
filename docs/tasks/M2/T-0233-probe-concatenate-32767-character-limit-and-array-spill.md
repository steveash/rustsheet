---
id: T-0233
title: "Probe CONCATENATE 32767-character limit and array spill values"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for CONCATENATE's 32767-character limit and for the element values of a spilled CONCATENATE (arrays.md entries marked UNVERIFIED), so the implementation can be confirmed or corrected.

## Context
Found by the T-0226 fidelity review. Use the `excel-probe` skill. Code: `functions/text/concatenate.rs`.

## Acceptance criteria
- [x] Probe cases (with room to spill) for 32767 vs 32768 chars, scalar and per-element, emoji (UTF-16 units), `CONCATENATE({"a";"b"},"x")`, `CONCATENATE(A1:A2,B1:B2)`, mismatched shapes, number/boolean in arrays; oracle values recorded.
- [x] arrays.md entry status updated; implementation fixed if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/concatenate-limit-spill` (68 cells, excel-graph 2026-10-06). Every spilled-element
question matches the implementation (broadcast, `#N/A` past a shorter argument, errors per element,
number/boolean text); arrays.md records the element values (VERIFIED). The 32,767 limit counts
UTF-16 code units: `truncate_text` and REPT counted chars/UTF-8 bytes (`REPT("😀",16383)` and
`REPT("é",20000)` were wrongly `#VALUE!`), fixed. Recorded in coercion.md. Eval: probes +68 cells,
61 pass; 7 new-coverage known failures (6 need LEN in UTF-16 units → T-0249; 3 of those also
involve a lone surrogate, which a Rust `String` cannot hold), 0 regressions, 0 fixed elsewhere.
Follow-up: T-0249.
