---
id: T-1301
title: "Probe ECMA.CEILING, port if Excel evaluates it as CEILING (ironcalc 8a82396a)"
milestone: M2
lane: functions-math
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`ECMA.CEILING` parses, round-trips and evaluates exactly as excel-graph does, or is shown to be unsupported there.

## Context
Upstream IronCalc 8a82396a (2026-09-23, first listed 2026-10-10) added `ECMA.CEILING`, evaluated
like the two-argument `CEILING`. The function is not in `eval/catalog/excel-functions.tsv`, and
upstream is not evidence. Probe it with the excel-probe skill first: positive and negative numbers
and significances, a zero significance, mixed signs (CEILING is #NUM! there), the one-argument form,
text and boolean arguments, and how FORMULATEXT prints it (any `_xlfn.` prefix on save). Port only
what excel-graph confirms (`Ported-From: ironcalc@8a82396afbababfdac296c70b02ac9bdb456db98` if code
is taken), reusing our CEILING implementation rather than copying it. Record the rule in
docs/excel-semantics/.

## Acceptance criteria
- [ ] A probe case covering the forms above is oracle-filled.
- [ ] The engine matches every cell (or `ECMA.CEILING` is left unknown if Excel rejects it); findings recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CEILING.MATH, CEILING.PRECISE, ISO.CEILING and the FLOOR family.

## Handoff (resumed)
Probe done (sonnet run, 2026-10-12): `probes/ecma-ceiling` and `probes/ecma-ceiling-formulatext` are oracle-filled and recorded in docs/excel-semantics/numbers.md ("ECMA.CEILING"). Excel has ECMA.CEILING with exactly CEILING's 2-argument semantics, stored as `_xlfn.ECMA.CEILING`, printed unprefixed by FORMULATEXT. Baseline `eval/baseline/probes.txt` already carries the 12 new known failures (via `make eval-baseline`).

Not done, because it needs a protected path: implement the function. `crates/rustsheet-engine/src/functions/mod.rs` is exactly at its oversized-file ratchet (2183 lines), so the new `Function::EcmaCeiling` variant, name-table row, language row and dispatch (about 4 lines, as in T-1214/#619) need a ratchet decision record (decision 0010 style). Also needed: a `ecmaceiling` field in `tools/upstream/generate_language` and `language/mod.rs`, a regenerated `language.bin`, `static_analysis.rs` (signature = `args_signature_scalars(arg_count, 2, 0)`, `scalar_arguments`), an `_xlfn.ECMA.CEILING` row in `functions/xlsx_names.rs`, and a dispatcher that reuses `fn_ceiling` (no copy). Then `make eval-baseline` to record the 12 fixes.
