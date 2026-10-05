# Numeric quirks re-audit after exact float parsing (T-0026, 2026-10-05)

Until T-0019 the eval harness read `*.oracle.json` numbers with serde_json's default float
parser, which can land 1 ulp off. Every rule in `docs/excel-semantics/numbers.md` and
`engineering.md` that cites oracle-sidecar values was re-checked here against the sidecars, read
with a correctly rounded parser. Python's `json` module parses numbers with a correctly rounded
`float()`, which is what the harness has done since T-0019 (test
`json_numbers_parse_to_the_exact_double` in `crates/rustsheet-eval/src/value.rs`). That makes it
equivalent to `rustsheet-eval show` for these values, and it can tally whole probe families in a
single pass.

Recipe, to reproduce: for each `eval/suites/probes-numeric/cases/<hash>.case.json`, read its
`hypotheses` map (cell → tag such as `snap-addsub/top-level-sub/base=3/sign=+1/k=5`). Look up
the cell's `{"t","v"}` in the matching `.oracle.json` under `sheets.<sheet>`, then group by tag
fields:
- **snap-addsub:** count `v == 0`.
- **compare-tolerance:** count the boolean `v`.
- **literal-15-digit:** compare `v` with the cell's literal cut to 15 significant digits using
  Python `decimal` `ROUND_DOWN`, and separately with `ROUND_HALF_UP`.

For the single cells in the table, read the cited cell and compare it with the stated double.

## Result: no rule changes

| Rule (numbers.md) | Evidence re-read | Why a 1-ulp misread could or could not matter | Re-check |
|---|---|---|---|
| Near-zero snapping | probes-numeric/30bc5f8fab02d0a2, 1,296 cells | Verdict is "result is exactly 0" (0 parses exactly) | k ≤ 3: 24/24; k = 4: 15/24; k = 5–7: 9/24; k = 8: 0/24 for `top-level-sub`, `add-negated`, `sum-fn`; 0/24 for k ≥ 1 in the parenthesized and follow-on variants. Matches. |
| Comparison tolerance | probes-numeric/99bc3137b5869417, 648 cells | Booleans | `=`, `<>`, `<=`, `>`: all 12 bases equal for k ≤ 2; 10/12 at k = 3–4, 9 at 5, 8 at 6–7, 7 at 8. `<`, `>=` uninformative. Matches. |
| 15-digit literal truncation | probes-numeric/23733e52a2ff91ab, 450 cells | Numeric: could flip truncate vs round | Truncation to 15 significant digits matches 450/450; round-half-up fails 171 of them, so the verdict is discriminating. Matches. |
| Number → text | probes-numeric/4f413cc257ce15c4 | Text results | Unaffected (strings, not parsed as floats). |
| ROUND decimal-correct | probes/precision-and-rounding A6–A8 | Numeric | 1.01, 2.68, -3 are the nearest doubles. Matches. |
| Subnormal → 0 | probes/precision-and-rounding A14 | Exact 0 | Matches. |
| SIN/COS at k·90° | poi/oracle/47813.oracle.json, 7 cells | Numeric | Values are exactly k·6.1257422745431E-17 for k = 1, 2, −3, −4, 5, 6, −7. Matches. |
| ACOS(0.5) | poi/oracle/48539.oracle.json Instance1!E23 | Numeric | 1.0471975511965976, not libm's …979. Matches. |
| `^` not correctly rounded | gen-m2 1b6ceed7489e0800 A62, f0d1bacec5de8146 A196 | Numeric | 0.7071067811865475 vs pow …476; 1.0000000000000005E+200 vs 1E+200. Matches. |

Not affected (evidence is xlsx cached values, which never went through the JSON parser):
engineering.md has no sidecar-float evidence and is unchanged. Also unaffected:
SINH/COSH/TANH including the T-0111 TANH 0.9 cutoff (poi FormulaEvalTestData_Copy.xlsx has no
oracle sidecar), MOD, POWER, BESSEL (engineering.md), GAMMALN. ROUND with huge num_digits is
already UNVERIFIED (T-0218). The criteria and date probes are integers, booleans and text.

## Engine fits

T-0914 (done) was fitted to xlsx cached values, so it is unaffected. T-0110 and T-0917 are not
implemented yet, and their probe evidence is confirmed above. No engine fit is contradicted, so no
task was filed.
