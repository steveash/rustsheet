---
id: T-0170
title: "Oracle fixture xlsx: LEN over a shared string with phonetic rPh runs"
milestone: M1
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that phonetic `<rPh>` runs in a shared string are not part of the cell text (LEN).

## Context
T-0164 excludes `<rPh>` from imported shared strings on ECMA-376 evidence alone (docs/excel-semantics/xlsx-storage.md, "Phonetic (furigana) runs are not cell text", status PLAUSIBLE). Spec cases cannot carry phonetic markup, so build a small fixture xlsx (see how other xlsx cases are filled via `rustsheet-oracle`/`rustsheet-eval cases`) with an `<rPh>`-annotated string and `=LEN(A1)`, and get Excel's value.

## Acceptance criteria
- [ ] A fixture xlsx case with oracle values for LEN/value of an rPh-annotated string; engine matches; xlsx-storage.md entry moves to VERIFIED (or is corrected).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
