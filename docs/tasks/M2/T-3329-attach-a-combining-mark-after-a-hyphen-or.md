---
id: T-3329
title: "Attach a combining mark after a hyphen or apostrophe to that skipped character, not to the letter before it (t3304 multi-letter-accents G4)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=`, `<`, `>` and the text criteria order a text with a combining mark right after a hyphen or
apostrophe as Excel does: the mark belongs to that skipped character, not to the letter before it.

## Context
- coercion.md, "Shared leading units are skipped before the sum", bullet "A mark after a hyphen or
  apostrophe goes with that skipped character" (T-3304, probes/t3304-collation-multi-letter-accents
  sheet G4, excel-graph 2026-10-31). Fitted rule: the mark adds nothing to the accent pass (`"a-"&UNICHAR(769)&"b"`
  has the accents of `"ab"`, so it is below `"áb"`, `"á-b"` and `"á'b"`), and in the skipped-character
  pass it raises the skipped character it follows: `"a'b"` < `"a-b"` < `"a'"&UNICHAR(769)&"b"` <
  `"a-"&UNICHAR(769)&"b"`. The smallest fit for the probed cells: the skipped character's weight
  plus the weights of the marks after it ([`APOSTROPHE`] 0x80 + 12 is above [`HYPHEN`] 0x81), with
  the position unchanged. Only U+0301 was probed, so keep the choice minimal and say so in the code.
- Ours: `accent_units` in crates/rustsheet-engine/src/functions/util/collation.rs appends a
  `Element::Mark` after `Element::Skipped` to the previous letter's run (`started` is already
  true), so `"a-"&UNICHAR(769)&"b"` = `"á-b"`; `skipped` drops marks. Change both, and `CollationKey`
  with them (texts equal under `cmp_text_pairwise` must keep equal keys).
- Failing cells (47):
  - probes/t3304-collation-multi-letter-accents.case.json G4!F11:I11, G4!K11, G4!M11, G4!O11, G4!R11, G4!T11,
    G4!F20:I20, G4!R20, G4!T20, G4!L5:L8, G4!L10, G4!L12, G4!L14, G4!L17, G4!L19, G4!U5:U8, G4!U17, G4!U19,
    Rank!D5:D8, Rank!D10:D12, Rank!D14, Rank!D17, Rank!D19:D20, Rank!H10:H12, Rank!H14, Rank!H19:H20

## Acceptance criteria
- [ ] Every cell of probes/t3304-collation-multi-letter-accents passes; `make eval-baseline` records it
      (0 regressions); the other collation probes stay green.
- [ ] A unit test in collation.rs pins `"a'b" < "a-b" < "a'"+U+0301+"b" < "a-"+U+0301+"b" < "áb"`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other marks or several marks after a skipped character, and marks after the en/em dash or control
  characters (unprobed; a probe task if needed).

## Outcome
`accent_units` drops a mark that follows a skipped character; `skipped` adds the marks after a skipped
character to its weight (U+0301 only probed). Unit test `mark_after_skipped_character_raises_it`.
Eval: 47 known failures fixed in probes (47 lines removed from eval/baseline/probes.txt), 0 regressions.
