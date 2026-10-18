---
id: T-1580
title: "Concatenation and 32,767-unit truncation keep a lone high surrogate (SPEC-0010 step 6)"
milestone: M2
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-1576, T-1578]
---

## Goal
`&`, CONCATENATE and the 32,767-unit truncation keep a lone high surrogate when the limit cuts a pair, and concatenation keeps lone halves.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md; docs/excel-semantics/coercion.md "Truncation inside a surrogate pair keeps a lone high surrogate". Replace `truncate_text`/`concat_text` (cast.rs) by `Text::truncate_utf16`/`Text::concat` in arithmetic.rs (`&`) and functions/text/concatenate.rs; CONCATENATE's argument loop appends with `Text::push` (not pairwise `concat`) so it stays linear; string operands of `&` are borrowed as `&Text`, not run through `cast_to_string`/`array_node_to_string` into a `String` (that would lose the lone half and add a copy), and checks the 32,767-unit limit before allocating. `concat` re-pairs a lone high with a following lone low surrogate (UTF-16's rule); that is provisional until T-1582 probes it, and is revisited if T-1582 disagrees. Failing cells: probes/concatenate-limit-spill E8, E9, E11; probes/text-surrogate-positions A6 once T-1579 has landed (E10 needs T-1581).

## Acceptance criteria
- [x] E8, E9, E11 pass; A6 passes if T-1579 is merged (else note it in the Outcome for T-1579's follow-up); they leave eval/baseline; no eval regressions.
- [x] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%.
- [x] coercion.md entry updated; decision 0033 mention in it says superseded.
- [x] `make check` green; reviewers approve.

## Out of scope
- CONCAT and TEXTJOIN truncation (unprobed; T-1582).

## Outcome
`&` and CONCATENATE now run on `Text`: `Text::concat` and `Text::truncate_utf16` replace `concat_text`/`truncate_text` (deleted); string operands of `&` are `Text` (`cast_to_text`, `array_node_to_text`, borrowed for array elements); CONCATENATE appends with `Text::push` and, once the limit has cut the text, reads the remaining arguments for errors without re-cutting. coercion.md updated (decision 0033 superseded). Bench `crates/rustsheet/benches/text.rs` (release, min of 11 runs, before -> after): `A1&"x"` 24.2 -> 23.6 ms, `MID(A1,2,3)` 22.2 -> 22.2 ms, `LEFT(A1,2)` 32.1 -> 32.8 ms (code unchanged; run-to-run noise about 5%), CONCATENATE(255 x 130 units) 73.0 -> 41.8 ms, `Text::push` lone first 54.6 -> 54.5 ms. Eval: see the PR (probes/concatenate-limit-spill E8, E9, E11 leave the baseline). A6 of text-surrogate-positions needs T-1579 (MID keeping the lone half) and E10 needs T-1581.
