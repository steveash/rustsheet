---
id: T-1796
title: "TEXTBEFORE/TEXTAFTER match_mode 1 may slice the original text at lowercased byte offsets (non-ASCII panic audit)"
milestone: M9
lane: functions-text
status: todo
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTBEFORE and TEXTAFTER with match_mode 1 never panic on non-ASCII text whose lowercase has a different byte length (CLAUDE.md directive 5), and fold case the way Excel does for these two functions.

## Context
Found by the T-1794 architecture review. `match_text` with `Case::Insensitive` (crates/rustsheet-engine/src/functions/text/util.rs ~150-163), used by TEXTBEFORE/TEXTAFTER (common.rs ~617 and ~722) and possibly `substitute`, lowercases with `to_lowercase()` and returns byte offsets into the lowercased string. If a caller slices the original with those offsets, text such as `"İİİ,a"` (U+0130 lowercases to two chars) gives an out-of-range or non-char-boundary slice: the same bug class T-1794 fixed in TEXTSPLIT. Not yet traced: confirm with a test first.
Excel's folding differs between functions: TEXTSPLIT matches `İ` to `i` and not the Kelvin sign to `k` (docs/excel-semantics/text.md "TEXTSPLIT match_mode 1 case folding"), SEARCH does the opposite (`position.rs` `fold_case`/`fold_char`). Probe TEXTBEFORE/TEXTAFTER before choosing a fold (excel-probe skill), and reuse `fold_char`/`fold_for_split` rather than adding a third fold; if a third consumer needs it, move the folds to one shared home in functions/text.

## Acceptance criteria
- [ ] A test reproduces the panic (or shows the offsets cannot misalign) for TEXTBEFORE and TEXTAFTER with match_mode 1 on `İ` text, and passes after the fix; every slice of the original goes through char-aligned offsets or `str::get`.
- [ ] A probe case records Excel's TEXTBEFORE/TEXTAFTER folding for `İ`/`i`, the Kelvin sign/`k`, `ß`/`ss` and `Ä`/`ä`, and the code matches it; findings are in docs/excel-semantics/text.md.
- [ ] `make eval`: 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTSPLIT (done in T-1794) and SEARCH/FIND.
