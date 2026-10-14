---
id: T-0915
title: "SIN/COS/TAN (and SEC/CSC/COT): Excel's argument reduction near multiples of pi"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1008]
---

## Goal
For arguments close to multiples of pi, Excel returns values that reflect a limited-precision pi in argument reduction (e.g. `SIN(PI())` is 1.22514845490862E-16 in Excel versus libm's 1.2246467991473532e-16). Reproduce Excel's results.

## Context
Kernel map (T-1522, decision 0056): rank 6 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Evidence: `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`, sheet SIN_COS_TAN, NumericMismatch cells B44, D44, C45, D45, C46, D46, B47, D47, C48, B49, D49 (inputs A44..A49 are near/at k*pi), plus NumericClose B34/C34/D34, C36/D36, C37/D37, and COT_CSC_SEC C34 (CSC) / SEC cells. Examples: B47 `SIN(A47)` expected 1.22514845490862e-16 got 1.2246467991473532e-16; C48 `COS(A48)` expected 6.1257422745431e-17 got 6.123233995736766e-17; D45 `TAN(A45)` expected 999849864538.9548 got 999849889614.2125.
Suspected cause: x87-style reduction using a pi approximated to ~66 bits (the ratio 1.22514845/1.22464679 ~ 1.0004 matches pi's rounding error at that precision). Hypothesis only: derive the effective pi constant that reproduces all cached values (fit against the 11 mismatch cells), then apply reduction `x - k*pi_eff` only when the argument is near a multiple of pi (check where Excel and libm agree for ordinary arguments so no currently passing cells regress).

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1008 sweep (`probes-numeric/m9-trig-reduction`), never blind.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/0e40208f51c2454c Calc!A3 `=TAN((Data!D6/(Data!C7*Data!A$9)))` = TAN(28000): Excel -1.611178907173344, engine (libm) -1.6111789071733424 (NumericClose). Not near a multiple of pi; libm `tan`, `sin/cos` and reduction by `fmod(x, 2*pi)` (-1.6111789071694178) all miss, so the reduction constant matters for ordinary large arguments too.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3b8e9078261b5bd2 Calc!A40 `=SIN(EDATE(54267,((-0.5)*(-1))))` (SIN(54267)): Excel -0.7652941001153616, engine -0.765294100115361.

## Acceptance criteria
- [ ] The 11 NumericMismatch SIN/COS/TAN cells above plus SEC/CSC cells on the same inputs leave eval/baseline/ironcalc-calc.txt, with no new F lines (NumericClose cells may remain; list them in Outcome).
- [ ] The 12 poi/47813.xlsx lines (Numbers!C92, B182, C272, B362, C452, B542, C632 at multiples of 90°, and the few-ulp
  cells B185, B299, C451, C683, B721 at 183/297/449/681/719°) leave eval/baseline/poi.txt (oracle: excel-graph sidecar).
- [ ] The reduction lives in one helper shared by SIN, COS, TAN, SEC, CSC, COT.
- [ ] Quirk and evidence recorded in docs/excel-semantics/numbers.md.
- [ ] `make check` green.

## Out of scope
Hyperbolic functions (T-0914).
