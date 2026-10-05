---
id: T-0012
title: "Catalogue existing directive-5 deviations as DEVIATION entries"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every place the engine deliberately differs from Excel for safety has a
`Status: DEVIATION (decision 0014)` entry in docs/excel-semantics/, so the canonical list matches the code.

## Context
Decision 0014 requires an entry per directive-5 deviation. Only the BESSELJ/Y order cap has one.
Known candidates: `MAX_SIZE = 1_000_000` in
`crates/rustsheet-engine/src/functions/math_and_trigonometry/array_size.rs` (SEQUENCE/RANDARRAY return `#ERROR!`,
not even an Excel error value, above 1M cells; Excel allows the full grid), `MAX_DEGREES_OF_FREEDOM` in `chisq.rs`, the 2^53 limit in
`gcd_lcm.rs`, parser/evaluator recursion limits, importer limits, and functions that would do I/O
in Excel (WEBSERVICE, external links). Some may be real Excel limits: those become VERIFIED rules
with oracle evidence (`excel-probe`), not deviations.

## Acceptance criteria
- [x] `rg` sweep for limit constants and I/O-function stubs, listed in the PR body.
- [x] Each true deviation has a DEVIATION entry stating Excel's behavior and ours, with an `Implemented in:` path and a code comment pointing back to it.
- [x] Each limit that matches Excel has a VERIFIED entry with oracle evidence (or an UNVERIFIED entry with a pending probe).
- [x] `make check` green; reviewers approve.

## Out of scope
Changing any limit's value.

## Outcome
Swept `rg 'const (MAX|LIMIT|CAP|DEPTH|ITER)…'` over the engine, xlsx and API crates, plus the
I/O-function stubs. New probe `probes/resource-limits` (14 cells, excel-graph 2026-10-05).

**DEVIATION entries** (each `Implemented in:` path has a code comment pointing back to the entry):
- array results above 1,000,000 cells are `#ERROR!` (arrays.md; Excel builds `SEQUENCE(1000,1001)`);
- the edit API refuses edits inside spill extents above 2^22 cells (arrays.md);
- hyperlink ranges above 10,000 cells link only their top-left cell (xlsx-storage.md).

The Bessel order cap and the parser depth limit already had entries.

**Excel's own limits** (VERIFIED):
- CHISQ.DIST df ≤ 10^10;
- BITAND < 2^48.

**Probes disproved two assumptions:**
- Excel accepts 2^53 in GCD/LCM, where we return `#NUM!` → follow-up T-0239.
- Concatenation truncates at 32,767 characters instead of returning `#VALUE!`, and `&` is unbounded in our engine → follow-up T-0240.

**Not deviations:**
- the evaluator's `DEFAULT_MAX_DEPTH`/`DEFAULT_MAX_STACK`: deep chains unwind and replay, so values are unchanged;
- `MAX_TERMS` and `*_LIMIT` constants: numeric algorithm thresholds;
- OCT/HEX bounds, the 31-character sheet name and the maximum date serial: Excel's documented ranges, already covered by passing ironcalc-calc cases;
- WEBSERVICE/FILTERXML: not in the engine's function list, so no I/O path exists (decision 0027 scopes WEBSERVICE out).

Eval: probes +14 cells, with 6 new known failures (4 by design, 2+2 for T-0239/T-0240); 0 regressions.
