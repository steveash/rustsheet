---
id: T-1483
title: "Spec: engine text that can hold lone surrogates (decision 0033 reopened)"
milestone: M2
lane: engine
status: in-progress
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
- [ ] docs/specs/SPEC-NNNN-*.md chooses a representation, with the evidence above and the
      boundaries (parser, xlsx, bindings) it touches.
- [ ] Implementation tasks (size S/M) are filed with `make new-task`, M2, and together cover all
      10 cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation itself.
