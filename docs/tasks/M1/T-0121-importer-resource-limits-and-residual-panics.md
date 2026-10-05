---
id: T-0121
title: "Importer: resource limits (decompressed size, string length, entity expansion) and residual panics"
milestone: M1
lane: engine
status: in-progress
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
- [ ] Typed `XlsxError` limit errors for decompressed part size, shared-string/cell text length and entity expansion; xmlbomb files rejected by the limit.
- [ ] The residual panics above replaced by errors, with tests.
- [ ] `make check` green; baseline only shrinks.

## Out of scope
What Excel does with an oversized shared string (needs an oracle probe, T-0004).
