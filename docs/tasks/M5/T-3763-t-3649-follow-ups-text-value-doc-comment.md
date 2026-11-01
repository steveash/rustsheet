---
id: T-3763
title: "T-3649 follow-ups: text-value doc comment, display color of a color-only section, test split"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The docs and tests left by T-3649 (PR #2780) say only what the evidence shows, and each rule has its own test.

## Context
These are the MINOR review findings on PR #2780, filed rather than fixed there to keep the reviewed head.
- `format_text_value`'s doc comment (crates/rustsheet-engine/src/formatter/text_value.rs) still says a text value under fewer than four sections is unprobed. probes/text-value-kind-sweep F2:G69 and probes/text-value-kind-sweep-b F2:G20 now cover it, so cite them.
- number-formats.md "A section of only a color or a condition prints General" says "The cell keeps the section's color". The probes test TEXT() strings only and never observe color. Mark that sentence inferred for cell display, or probe the displayed color (desktop question, decision 0039).
- `trailing_separator_and_bracket_only_sections` in formatter/test/test_condition.rs mixes two rules. Split it into one test per rule, and move the text-value assertions next to the other text-value tests. Wrap the long line in that number-formats.md entry.
- The bracket-only arm in `format_number` applies the section's numeral system (`p.numerals`, from the T-3532 merge). No probe covers a color or condition section that also carries a numeral-system tag (`[Red][$-2010000]`). Say so in the entry, or add a probe cell.
- A trailing `;` now adds an empty part, so `parts.last()` for `x@;` / `@;` is that empty section. Cell display of those shapes is unprobed. Add cells for them to a probe (TEXT and display).

## Acceptance criteria
- [ ] The doc comment cites the probes; no "unprobed" claim remains for one to three sections.
- [ ] The color sentence is marked inferred, or backed by a probe.
- [ ] The numeral-tag combination and the `x@;` shapes are probed, or marked unprobed in number-formats.md.
- [ ] One test per rule; behavior unchanged (no eval change).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change.
