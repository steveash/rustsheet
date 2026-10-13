---
id: T-0414
title: "xlsx import indexes shared strings instead of a linear search per inline string (port ironcalc 31b0c731)"
milestone: M11
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Loading a sheet with many inline-string or formula-string (`t="str"`) cells is linear in the number of cells, not quadratic.

## Context
Upstream IronCalc 31b0c731 (2026-10-08) replaced the linear `shared_strings.iter().position(...)`
search done per inline/formula string cell with a table that keeps a HashMap from string to first
index (first occurrence still wins, so indices are unchanged). Our importer has the same two linear
searches in `crates/rustsheet-xlsx/src/import/worksheets.rs` (around lines 438 and 463). Workbooks are
untrusted input (directive 5): a crafted sheet with many distinct inline strings makes load time
quadratic. Port it with `Ported-From: ironcalc@31b0c731c8e27c9e8ed2751a7fbc572c13b0c14a`. Reuse an
existing hasher (`KeyHasher`, T-0932) if it fits instead of adding a new one.

## Acceptance criteria
- [ ] Shared-string lookups during import are O(1) on average; indices are unchanged (import tests and eval show 0 regressions).
- [ ] A test with many distinct inline strings loads in bounded time.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other import performance work.

## Outcome
Added `SharedStringTable` (strings plus a first-index `HashMap`) in `import/shared_strings.rs`; the
`t="str"` and `inlineStr` paths call `intern` instead of a linear `position` search. Indices are
unchanged (first occurrence wins). Std `HashMap` (SipHash) kept because untrusted keys; `KeyHasher`
is a u64 integer hasher and does not fit strings. Eval delta: none.

Ported-From: ironcalc@31b0c731c8e27c9e8ed2751a7fbc572c13b0c14a
