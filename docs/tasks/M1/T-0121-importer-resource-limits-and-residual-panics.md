---
id: T-0121
title: "Importer: resource limits (decompressed size, string length, entity expansion) and residual panics"
milestone: M1
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-xlsx` import enforces its own resource bounds and has no remaining panics on malformed parts.

## Context
Found by T-0118 reviewers. The 32,767-char shared-string cap lives only in the eval snapshot reader (`crates/rustsheet-eval/src/xlsx_snapshot.rs`); the importer has no zip-entry/decompressed-size, shared-string or entity-expansion limit. Residual panics: `n.text().unwrap()` on an empty `<t/>` in comments (`import/worksheets.rs` ~255), `v[1]` after `split("/worksheets/")` (~593), and `pattern_fill[0]` in `import/styles.rs` ~235 (verify guarded). Add a test that an empty `<fonts>` plus an `xf` with a font id imports without panic, and one that stripping `<cellStyles>` still imports.

## Acceptance criteria
- [x] Typed `XlsxError` limit errors for decompressed part size, shared-string/cell text length and entity expansion; xmlbomb files rejected by the limit.
- [x] The residual panics above replaced by errors, with tests.
- [x] `make check` green; baseline only shrinks.

## Out of scope
What Excel does with an oversized shared string (needs an oracle probe, T-0004).

## Outcome
`rustsheet-xlsx` reads every part through `import/package.rs` (`Package`), which charges the bytes
actually inflated against one budget (`ImportLimits`, default 256 MiB / 5,000 entries / 32,767
characters, the facade's numbers, which the facade now passes through). New typed
`XlsxError::Limit { limit: ImportLimit, max }` for zip entries, inflated bytes, shared-string/cell
text length (counted after `_xHHHH_` decoding) and entity expansion (declared entities are never
expanded; a billion-laughs part is rejected). Panics fixed: empty `<t/>` in a comment, a worksheet
outside `xl/worksheets/` (`v[1]`), and `replace_range(..2)` on short targets; relationship targets
now resolve per OPC (`resolve_target`). `pattern_fill[0]` and the other `[0]` indexes were already
length-guarded. Tests: empty `<fonts>` + xf font id, stripped `<cellStyles>`, all limits. The eval
snapshot reader reuses the text limit. Eval: 0 fixed, 0 regressions in every suite (poi included).
DEVIATION recorded in docs/excel-semantics/xlsx-storage.md. Follow-up: T-0163 (drop the facade's
zip pre-pass, which now inflates a second time).
