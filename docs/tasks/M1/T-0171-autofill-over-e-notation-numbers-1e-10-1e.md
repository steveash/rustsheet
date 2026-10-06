---
id: T-0171
title: "Autofill over E-notation numbers (1E-10, 1E+20) copies instead of extending: needs Excel evidence"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Autofill of a numeric pair written in E-notation (1E-10, 2E-10; 1E+20, 2E+20) matches Excel's fill behavior.

## Context
Found by T-0168: `UserModel::auto_fill_rows` over cells holding 1E-10 and 2E-10 (or 1E+20, 2E+20) copies the pair instead of extending it (pinned by `exponent_notation_sources_are_copied_not_extended` in test/user_model/test_autofill_extreme_steps.rs), because the number detector in user_model/sequence_detector.rs only sees the cell text. I believe Excel extends these to 3E-10, 4E-10, but have no evidence: the Graph oracle has no fill-handle operation, so this needs a desktop-Excel observation or a cached-value workbook. Do not implement from memory (CLAUDE.md directive 1). Output text is fine: it is typed back in and parsed (T-0143).

## Acceptance criteria
- [ ] Excel evidence for the fill over E-notation sources is recorded in docs/excel-semantics; the engine matches it and the characterization test is replaced.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
