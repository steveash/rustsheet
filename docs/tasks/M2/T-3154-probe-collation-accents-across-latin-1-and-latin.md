---
id: T-3154
title: "Probe collation accents across Latin-1 and Latin Extended-A on one letter, stacked combining marks, and Cyrillic ё/й as accent or letter"
milestone: M2
lane: research
status: done
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
- [x] A probe case ranks `à â ã ä å ă ā ą` with `a`, `è ê ë ė ě ē ę` with `e`, `ò ô õ ö ő ō`
      with `o`, `ù û ü ů ū ű ų` with `u`, and the same letters as the first of two-letter strings.
- [x] It ranks stacked marks (`"á"&UNICHAR(780)`, `"a"&UNICHAR(769)&UNICHAR(780)`,
      `"a"&UNICHAR(780)&UNICHAR(769)`, a mark after `ǎ`, a mark outside the 13 mapped ones such as
      U+0323) and `"йа"` against `"иб"`, `"ёа"` against `"еб"`.
- [x] It checks the final-sigma fold beyond exact MATCH: `COUNTIF(r,"ς*")`, `MATCH("ς*",r,0)`,
      `XMATCH("ς",r,2)`, `VLOOKUP("ς?",r,1,FALSE)` and a D-function criterion `ς` over `σx`, `ςx`, `Σ`.
- [x] It ranks Latin Extended-B letters (`ǎ`, `ș`, `ț`, `ơ`, `ư`) against their base letters and
      their combining spellings (U+0326, U+031B), and XMATCH/VLOOKUP with a decomposed key.
- [x] `collation.rs` and coercion.md follow the evidence; every collation probe still passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Scripts other than Latin, Greek and Cyrillic.

## Outcome
- Three probes (excel-graph): probes/t3154-collation-accents (595 cells: the accent order on
  `a e o u`, stacked and unmapped marks, `ё`/`й`, final sigma in wildcards and D-functions, Latin
  Extended-B and decomposed lookup keys), probes/t3154-collation-accent-weights (357 cells: 119
  spellings of `a` with 1 to 100 marks) and probes/t3154-collation-accent-pairs (812 cells: every
  pair's `<`, `=`, `>`).
- Finding: Excel's accent is the Windows sorting weight table's diacritic weight, summed over the
  combining marks in a wrapping byte (`a`+acute+acute = `å`; two perispomeni = one cedilla); the
  superscripts `ª º ¹ ² ³` differ in a later case-like pass; `ё`/`й` are accented letters; the
  final-sigma fold holds in wildcards and D-function criteria; XMATCH/XLOOKUP/COUNTIF equate
  decomposed keys, MATCH/VLOOKUP do not. Excel's comparison is not a total order (prefix rule,
  long mark runs): left to T-3181.
- Implemented: `collation_tables.rs` (`MARK_WEIGHT` from the Windows table, `DECOMPOSED` extended
  to Latin Extended-B and Additional, 241 letters) and `collation.rs` (accent byte, variant pass).
  coercion.md "Accent weights add up and wrap" records it.
- Eval delta: 3 new probe cases, 1764 cells, 1703 pass; 61 new-coverage known failures (55 in
  accent-weights, 6 in accent-pairs, all T-3181); no regressions; every earlier collation probe
  still passes.
