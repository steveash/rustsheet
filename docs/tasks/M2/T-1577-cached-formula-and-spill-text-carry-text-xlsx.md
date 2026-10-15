---
id: T-1577
title: "Cached formula and spill text carry Text; xlsx writer escapes lone units (SPEC-0010 step 3)"
milestone: M2
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-1576]
---

## Goal
Cached formula and spill results keep a lone surrogate, and the xlsx writer writes it as Excel does (`_xHHHH_`).

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md "Where `Text` replaces `String`" and "Boundaries / xlsx writer". `FormulaValue::Text` and `SpillValue::Text` (types.rs) carry `Text`. Excel writes a lone unit of a cached value as `_xDE00_` (the oracle file of probes/text-surrogate-positions A6 holds `a😀_xDE00_b`). Add one public, documented helper in rustsheet-xlsx export/escape.rs that writes a `Text` segment by segment (`escape_xml` for `Str`, upper-case `_xHHHH_` for `Lone`); T-1578 reuses it for the harness.

## Acceptance criteria
- [x] `FormulaValue::Text(Text)`, `SpillValue::Text(Text)`; the bitcode format change is covered by a test that bytes from the old layout are an `Err` from `Model::from_bytes`, not a panic.
- [x] xlsx writer test: a cached value with a lone U+DE00 is written as `_xDE00_`, and an `_x` lookalike in a well-formed segment is still escaped as `_x005F_`.
- [x] Zero eval delta.
- [x] `make check` green; reviewers approve.

## Out of scope
- Reading lone surrogates from xlsx (non-goal of the spec).

## Outcome
`FormulaValue::Text` and `SpillValue::Text` carry `Text`; importers, API value and comparisons adapted. rustsheet-xlsx `export::escape::escape_text` writes a `Text` segment by segment (`escape_xml` for well-formed runs, upper-case `_xHHHH_` for a lone unit); the three cached-text writers use it. Tests: pre-change bitcode bytes are an `Err` from `from_bytes`; a bitcode round trip keeps a lone unit; a saved workbook holds `_xDE00_` and `_x005F_` lookalikes. Zero eval delta. T-1578 reuses `escape_text` (make it re-exported if the harness needs it; the helper is `pub` in the private `export::escape` module).
