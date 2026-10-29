---
id: T-3154
title: "Probe collation accents across Latin-1 and Latin Extended-A on one letter, stacked combining marks, and Cyrillic ё/й as accent or letter"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3114]
---

## Goal
The accent order between the Latin-1 accents and the Latin Extended-A ones, the weight of a second
combining mark, and whether Cyrillic `ё`/`й` are accented letters are backed by oracle cells, and
`collation.rs` matches them.

## Context
- T-3114 (probes/t3114-collation-beyond-latin1) ranked accents only within the base letters Latin
  Extended-A uses, so these choices in `collation.rs` are UNVERIFIED (coercion.md "Letters beyond
  Latin-1 and combining marks"):
  - breve, dot above and caron sort after tilde, and ring after macron: never ranked against grave,
    circumflex or tilde on one letter;
  - a mark after an accented letter, or a second mark, adds its own accent weight (only `ΐ` is
    probed); marks other than the 13 mapped ones get `OTHER_MARK`;
  - `ё` and `й` are `е` and `и` plus an accent (their canonical decompositions), but single letters
    cannot tell that from separate letters.
- Method as in T-3114: a Data column ranked with `SUMPRODUCT(--(range<x))` and `=` counts
  (excel-probe skill, excel-graph).

## Acceptance criteria
- [ ] A probe case ranks `à â ã ä å ă ā ą` with `a`, `è ê ë ė ě ē ę` with `e`, `ò ô õ ö ő ō`
      with `o`, `ù û ü ů ū ű ų` with `u`, and the same letters as the first of two-letter strings.
- [ ] It ranks stacked marks (`"á"&UNICHAR(780)`, `"a"&UNICHAR(769)&UNICHAR(780)`,
      `"a"&UNICHAR(780)&UNICHAR(769)`, a mark after `ǎ`, a mark outside the 13 mapped ones such as
      U+0323) and `"йа"` against `"иб"`, `"ёа"` against `"еб"`.
- [ ] `collation.rs` and coercion.md follow the evidence; every collation probe still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scripts other than Latin, Greek and Cyrillic.
