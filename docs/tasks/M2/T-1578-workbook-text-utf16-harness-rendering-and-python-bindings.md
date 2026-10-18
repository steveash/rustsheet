---
id: T-1578
title: "Workbook::text_utf16, harness rendering and Python bindings for lone surrogates (SPEC-0010 step 4)"
milestone: M2
lane: api
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-1577]
---

## Goal
A lone surrogate in a cell value reaches the public API, the eval harness and Python exactly.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md "Boundaries". `rustsheet::Value::Text(Arc<str>)` stays the lossy view; add `Workbook::text_utf16(r) -> Result<Option<Arc<[u16]>>, RefError>` (`Some` exactly when the value is text holding a lone surrogate). The harness (crates/rustsheet-eval/src/engine.rs) renders such text with the T-1577 helper's `_xHHHH_` form, matching the snapshot reader, which keeps an undecodable `_xDE00_` literally. The Python facade (bindings/python/src/facade.rs) builds the `str` from the units with `surrogatepass` when the lossy text contains U+FFFD and `text_utf16` is `Some`.

## Acceptance criteria
- [x] `Workbook::text_utf16` with doc comment and unit test.
- [x] Harness renders lone units as `_xHHHH_` (unit test with a hand-built value).
- [x] Python `value` builds the `str` with `surrogatepass`; partly met: only a Rust unit test of the byte form, the Python-level test (`'\ude00'`) needs a formula that produces a lone surrogate, so it is T-1603.
- [x] Zero eval delta (no engine function produces a lone surrogate yet).
- [x] `make check` green; reviewers approve.

## Out of scope
- Python or Rust *inputs* holding lone surrogates.

## Outcome
- `Workbook::text_utf16`, harness `_xHHHH_` rendering and Python `Workbook.value` for lone surrogates.
- The `_xHHHH_` helper `lone_escape` lives in `rustsheet_engine::text` (not xlsx `export/escape.rs` as SPEC-0010 says), so the xlsx writer and the harness share one definition.
- The Python-level test and `values()` are T-1603 (depends on T-1579).
- Eval delta: none (no engine function produces a lone surrogate yet).
