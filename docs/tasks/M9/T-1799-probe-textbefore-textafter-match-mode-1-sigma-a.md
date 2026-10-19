---
id: T-1799
title: "Probe TEXTBEFORE/TEXTAFTER match_mode 1 sigma: a same-case Σ delimiter and a Σ before a space, digit or punctuation"
milestone: M9
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTBEFORE/TEXTAFTER match_mode 1 sigma folding matches Excel for the two shapes T-1796 left unprobed, with the evidence recorded.

## Context
Found by the T-1796 excel-fidelity review. `fold_for_before_after` (crates/rustsheet-engine/src/functions/text/casefold.rs) folds the text and the delimiter separately and by position: a `Σ` followed by a letter is `σ`, any other `Σ` is `ς`. Two consequences have no oracle cell (probes/textbefore-casefold-length covers A1-A25):
- `TEXTBEFORE("aΣb","Σ",1,1)`: the text's `Σ` folds to `σ` and the lone delimiter `Σ` to `ς`, so we give `#N/A` although the case is identical.
- A `Σ` followed by a space, digit or punctuation, e.g. `TEXTBEFORE("ΟΔΟΣ ΚΑΙ","ς",1,1)` and `TEXTBEFORE("ΟΔΟΣ1","σ",1,1)`; only a `Σ` at the end of the text is probed (A25).
docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER match_mode 1 case folding" lists both as UNVERIFIED.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) covers both shapes for TEXTBEFORE and TEXTAFTER, and docs/excel-semantics/text.md records the result; the UNVERIFIED notes it answers are rewritten.
- [ ] If Excel differs, `fold_for_before_after` is changed to match and the probe passes; `make eval`: 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTSPLIT's and SEARCH's sigma (their own tasks if they differ).
