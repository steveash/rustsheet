---
id: T-1580
title: "Concatenation and 32,767-unit truncation keep a lone high surrogate (SPEC-0010 step 6)"
milestone: M2
lane: engine
status: todo
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
- [ ] E8, E9, E11 pass; A6 passes if T-1579 is merged (else note it in the Outcome for T-1579's follow-up); they leave eval/baseline; no eval regressions.
- [ ] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%.
- [ ] coercion.md entry updated; decision 0033 mention in it says superseded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CONCAT and TEXTJOIN truncation (unprobed; T-1582).
