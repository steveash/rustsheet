---
id: T-1483
title: "Spec: engine text that can hold lone surrogates (decision 0033 reopened)"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A spec (docs/specs/) for engine text that can hold an unpaired UTF-16 surrogate, as Excel text
can, split into implementation tasks that close the lone-surrogate known failures.

## Context
Decision 0052 reopened decision 0033: RustSheet may deviate from Excel only for directive 5, and
"a Rust `String` cannot hold a lone surrogate" is not a directive-5 reason. Excel keeps the lone
half when a cut splits a surrogate pair:
- docs/excel-semantics/text.md "MID and REPLACE positions inside a surrogate pair keep the lone
  half": probes/text-surrogate-positions A6, A11, A12, A29, A38, B10;
- docs/excel-semantics/coercion.md "Truncation inside a surrogate pair keeps a lone high
  surrogate": probes/concatenate-limit-spill E8-E11.
These 10 cells are M2 exit scope (decision 0051). Today `cast::truncate_text` and
`functions/text/util.rs` `utf16_slice` drop the straddling pair whole.

The spec should weigh at least: WTF-8 or UTF-16 text in the engine's value type; a narrower
representation that only text values need (parser, xlsx reader/writer and bindings keep `String`
at their boundaries); how UNICODE, CODE, comparisons, LEN/LEFT/RIGHT, `&` and the xlsx writer see
a lone surrogate (Excel: UNICODE of a lone high surrogate is `#VALUE!`, of a lone low one its
code); what the Python bindings return; and the cost on the text-heavy hot paths.

## Acceptance criteria
- [x] docs/specs/SPEC-NNNN-*.md chooses a representation, with the evidence above and the
      boundaries (parser, xlsx, bindings) it touches.
- [x] Implementation tasks (size S/M) are filed with `make new-task`, M2, and together cover all
      10 cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- The implementation itself.

## Outcome
docs/specs/SPEC-0010-text-with-lone-surrogates.md: engine text becomes `enum Text { Utf8(String),
Lone(Box<LoneText>) }` (24 bytes like `String`, so `ArrayNode`/`CalcResult` keep their sizes), with
a lossy `Deref<Target = str>` view for code not yet lone-aware and exact units for the rest;
boundaries: xlsx writer `_xHHHH_`, `Workbook::text_utf16`, harness rendering, Python
`surrogatepass`. Tasks filed (M2): T-1575 (type, opus), T-1576 (CalcResult/ArrayNode migration),
T-1577 (cached values + xlsx writer), T-1578 (API/harness/bindings), T-1579 (MID/REPLACE: A11,
A12, A29, B10), T-1580 (`&`/truncation: E8, E9, E11, A6), T-1581 (UNICODE/LEFT/RIGHT: A38, E10),
T-1582 (probe of the unprobed list). New quirk recorded: Excel writes a lone surrogate of a cached
value as `_xHHHH_` (xlsx-storage.md). Eval delta: none (docs only).
