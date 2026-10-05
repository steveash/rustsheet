---
id: T-0026
title: "Re-audit numeric quirks recorded before oracle floats parsed exactly"
milestone: M0
lane: research
status: in-progress
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
- [ ] Each numbers.md/engineering.md rule citing probes or probes-numeric evidence is re-checked with
      `rustsheet-eval show` on the current harness; counts and thresholds updated, or the rule marked confirmed.
- [ ] Any engine fit that the corrected values contradict is filed as its own task (no engine changes here).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes; new probes (file tasks instead).
