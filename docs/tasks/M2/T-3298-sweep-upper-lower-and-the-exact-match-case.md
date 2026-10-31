---
id: T-3298
title: "Sweep UPPER, LOWER and the exact-match case fold over every cased character, to find where Excel's case tables differ from the Rust std Unicode mappings the engine uses"
milestone: M2
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence (excel-graph) shows what UPPER, LOWER and MATCH's exact case-insensitive comparison do for every
character with a simple case mapping, not only the roughly 120 characters probed so far. Each block where Excel
differs from the engine is stated in docs/excel-semantics/ and owned by a task.

## Context
- Gap triage 2026-10-30e. Every visibly-off M0-M2 cell has an open owner and gen-m2 is at 100%, so the open M2 risk
  is in rules the engine extends by inference. Two M2 rules depend on one per-character case table:
  - UPPER and LOWER (text.md "UPPER, LOWER and PROPER: one-to-one case mapping, final-sigma context (T-1747)").
    The evidence is probes/case-mapping-expanding, 81 cells.
  - MATCH(..,0), VLOOKUP(..,FALSE) and HLOOKUP(..,FALSE) exact keys, and wildcard criteria (lookup.md "MATCH,
    VLOOKUP and HLOOKUP exact-match keys fold case one character to one (T-3133)"). The evidence is
    probes/t3133-lookup-criteria-case-fold, 35 characters. That section says: "UNVERIFIED: characters outside the
    list above are assumed to follow the same fold."
- The engine takes every mapping from Rust's `char::to_lowercase` / `to_uppercase` (rustc 1.97, whose Unicode
  version is recent) through `fold_char` (crates/rustsheet-engine/src/functions/util/collation.rs) and
  `lower_case` / `upper_case` (functions/text/string_format.rs). Excel's case tables come from Windows and may be
  older or tailored. The likely differences are mappings added in later Unicode versions:
  - Georgian Mtavruli U+1C90-U+1CBF (Unicode 11);
  - Cherokee small letters U+AB70-U+ABBF and U+13F8-U+13FD (Unicode 8);
  - Latin Extended-D additions U+A7C0-U+A7DC (Unicode 12-16) and Latin Extended-E;
  - Glagolitic, Coptic and Armenian;
  - the supplementary-plane cased scripts (Deseret U+10400, Osage U+104B0, Old Hungarian U+10C80, Warang Citi
    U+118A0, Medefaidrin U+16E40, Adlam U+1E900, Vithkuqi U+10570, Garay). These are also a surrogate-pair
    question: does Excel case-map a UTF-16 pair at all?
- Design for the probe (this task settles it and records it in the case description). Build the cells from
  UNICHAR so the workbook is plain ASCII. Pack many characters per cell, for example one cell per block of up to 64
  pairs, so the sweep stays within a few hundred cells:
  - `UPPER(lowers)` and `LOWER(uppers)`, each compared with EXACT against the expected string, plus the raw text, so a
    failing cell names the characters;
  - for the fold, `SUMPRODUCT(--ISNUMBER(MATCH(lower_i, upper_column, 0)))` style counts over Data columns that hold
    each pair, with per-character cells only where a block disagrees.
  Generating the case with a script in tools/ is fine. The list of cased characters comes from the engine's tables
  (or Python's `unicodedata`, for the candidate list only; Excel decides).
- Expected values that look wrong are still Excel's. A block Excel leaves unmapped is a quirk to record, not a bug.

## Acceptance criteria
- [ ] A new probes case (or a few, by plane) `probes/t3298-case-mapping-sweep*` with excel-graph oracle sidecars. It
      covers every BMP and supplementary character whose Rust simple lowercase or uppercase differs from itself, for
      UPPER, LOWER and the MATCH exact fold.
- [ ] text.md (UPPER/LOWER) and lookup.md (the T-3133 section) state Excel's tables by block, citing cells, and the
      lookup.md UNVERIFIED line is replaced by the observed rule.
- [ ] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions). Each is owned by a task filed in this PR (`functions-text` for UPPER/LOWER, `engine` for
      `fold_char`), and `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Collation order (coercion.md; T-3275, T-3276 cover other scripts and width or kana).
- PROPER's word-start rule and the final-sigma context (verified by T-1307 and T-1747).
- SEARCH, TEXTBEFORE and TEXTSPLIT folds beyond what the shared `fold_char` gives (follow-ups if the sweep shows
  that they differ).
- Fixing the engine (the owning tasks).
