---
id: T-1794
title: "TEXTSPLIT match_mode 1 can panic on non-ASCII text whose lowercase changes byte length"
milestone: M9
lane: functions-text
status: todo
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT never panics on any text and delimiters, including match_mode 1 with non-ASCII text (CLAUDE.md directive 5).

## Context
Found by the T-1781 security review (pre-existing). With match_mode 1, `fn_textsplit` sets `cmp_text = text.to_lowercase()`, which can change byte lengths for non-ASCII text ("İ" lowercases to "i" + U+0307). The byte offsets found in the lowercased copy are then used to slice the original text (textsplit.rs, around the `split_by_any` calls near lines 114-136 and 283-287), so `=TEXTSPLIT("İİİ,a",",",,,1)` may slice out of range or off a char boundary and panic. The code comment claims 1:1 byte mapping only for ASCII delimiters, not for the text.

## Acceptance criteria
- [ ] A unit test reproduces the panic first (`"İİİ,a"` with match_mode 1, and a delimiter that lowercases to a different length).
- [ ] TEXTSPLIT matches case-insensitively with char-aware offsets or `str::get`, never an unchecked slice; no panic for any input.
- [ ] Results for ASCII text and for text whose lowercase keeps its byte length are unchanged (eval: 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's exact case folding rules beyond what existing eval cases show (probe separately).
