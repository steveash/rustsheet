---
id: T-0026
title: "Re-audit numeric quirks recorded before oracle floats parsed exactly"
milestone: M0
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Every numeric rule and fit in docs/excel-semantics/ that was derived from oracle-sidecar values is re-checked
against the values as they really are, and corrected where it was built on a 1-ulp misread.

## Context
Until T-0019 the harness parsed `*.oracle.json` numbers with serde_json's default float parser, which can be 1 ulp
off (`crates/rustsheet-eval/src/value.rs` test `json_numbers_parse_to_the_exact_double`). Fixing it moved
probes-numeric from 60.19% to 68.08% bit-exact and removed ~294 false POI failures. Expected values in
ironcalc-calc come from xlsx cached values (not JSON), so they were unaffected; probes, probes-numeric and smoke
sidecars were. Candidates: numbers.md "SINH / COSH / TANH are exp-based" (the 0.9 cutoff and the COTH/CSCH/SECH
libm choice), "Near-zero snapping" and "Comparison tolerance" thresholds (k-counts per base), and any engine
code fitted to probes-numeric (T-0110, T-0914, T-0917 outcomes).

## Acceptance criteria
- [x] Each numbers.md/engineering.md rule citing probes or probes-numeric evidence is re-checked with
      `rustsheet-eval show` on the current harness; counts and thresholds updated, or the rule marked confirmed.
- [x] Any engine fit that the corrected values contradict is filed as its own task (no engine changes here).
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes; new probes (file tasks instead).

## Outcome
- **Method:** every numbers.md and engineering.md rule citing oracle-sidecar evidence was
  re-checked with a correctly rounded float parser. Details are in
  `docs/research/numeric-reaudit-2026-10-05.md`.
- **Rules:** all hold; each re-checked entry in numbers.md says so. Snapping and comparison counts
  are unchanged, because those verdicts are exact zeros and booleans. Literal truncation matches
  450/450, and 171 of those cells rule out rounding. The SIN/COS, ACOS and `^` values are exact.
- **Engine fits:** none contradicted, so no tasks were filed. T-0914 was fitted to xlsx cached
  values, which never went through the JSON parser.
- **Eval:** unchanged (docs only).
