---
id: T-3381
title: "Probe the unprobed kana and width collation choices of T-3321"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (an excel-probe case) for the collation choices T-3321 made without evidence, so
each one is either confirmed or filed as a fix task.

## Context
- docs/excel-semantics/coercion.md, T-3275 section, "Engine choices, UNVERIFIED" (T-3321).
  Choices to probe: the voicing weight against the Latin marks (`"a゛"` vs `"á"`, `"ª"`); the
  full-width variant against the superscript one (`"１"` vs `"¹"`); `ー` after `ん`, after an
  ideograph, after a digit or symbol; `ゝ`/`々` after a Latin letter; `ヷ ヸ ヹ ヺ`, `ゕ ゖ`,
  `ヽ ヾ`; `＇ －` against `– —`.
- MATCH/XMATCH/COUNTIF with wildcards and `ー`/`ゝ`/`々` after a Latin letter (`COUNTIF(r,"aー*")`,
  `COUNTIF(r,"*ー*")`, `MATCH("aー*",r,0)`, `XMATCH("aー*",r,2)` over `"aa"`, `"aー"`, `"々木"`):
  the engine keeps the plain case fold there (only literal exact MATCH/VLOOKUP/HLOOKUP use
  `collation::fold_exact_lookup`).
- Exact MATCH's mark fold (`collation::fold_exact_lookup`) generalizes the two probed cases
  (`MATCH("aー")` finds `"aa"`, a lone iteration mark finds `"ー"`): `ー` after any non-kana
  letter or digit, a leading mark in a longer text, and the fold on stored values too. Probe e.g.
  `MATCH("1ー",{"11"},0)`, `MATCH("αー",{"αα"},0)`, `MATCH("aa",{"aー","aa"},0)`,
  `MATCH("ゝa",{"ーa"},0)`.
- Also unprobed: the half-width `ｰ` (lone, after a Latin letter, against `ー`; the engine makes
  `"ー" = "ｰ"`, `"aー" = "aｰ"`); VLOOKUP/HLOOKUP sharing the exact-MATCH fold
  (`VLOOKUP("aー",{"aa",1},2,FALSE)`); how SORT/SORTBY and approximate MATCH order the
  non-transitive texts (`SORT({"aー";"ab";"aa";"Ａa"})`; the engine sorts `"aー"` as `"aa"`).
- Code: crates/rustsheet-engine/src/functions/util/collation.rs (`resolve`, `kana`,
  `fold_exact_lookup`).

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers every choice above, with oracle values.
- [ ] coercion.md marks each choice VERIFIED or records the observed behavior; mismatches are
      filed as engine tasks (or fixed in this PR when trivial).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing fixes beyond trivial ones: file engine tasks for mismatches.
