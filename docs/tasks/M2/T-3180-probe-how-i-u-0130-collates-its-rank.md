---
id: T-3180
title: "Probe how İ (U+0130) collates: its rank among i and the accented i's, and = / COUNTIF equality with i+U+0307 and I"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
How Excel's collation treats `İ` (U+0130) in `=`, `<`/`>`, COUNTIF and SORT is backed by an oracle case, and
`collation.rs` matches it.

## Context
- T-3133 (probes/t3133-lookup-criteria-case-fold Calc!J1, J3, J4, J33) showed that XMATCH's exact mode equates `İ`
  with `i` + U+0307 and not with `I` or `i`. `decompose` (functions/util/collation.rs) now gives `İ` that
  decomposition, so `İ` ranks as `i` with the dot-above accent. That rank, and that `=` and COUNTIF agree with XMATCH,
  are inferred and marked UNVERIFIED in docs/excel-semantics/coercion.md ("Letters beyond Latin-1 and combining
  marks").
- Method: the `excel-probe` skill, excel-graph, as in probes/t3114-collation-beyond-latin1. Rank `İ`, `"İa"`, `"İz"`
  in a Data column with `i í ì î ï ı I ia iz` and `"i"&UNICHAR(775)` using `SUMPRODUCT(--(range<x))`, and add cells
  for `"İ"="i"&UNICHAR(775)`, `"İ"="I"`, `"İ"="i"`, COUNTIF(range,"İ") and COUNTIF(range,"i"&UNICHAR(775)).

## Acceptance criteria
- [ ] A probe case with the cells above is committed with its excel-graph sidecar, and every cell passes.
- [ ] coercion.md records the result as VERIFIED, citing the case id and cells; `collation.rs` is changed only if the
      evidence requires it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The one-to-one fold of MATCH/VLOOKUP and wildcard criteria (T-3133, verified).
