# Numbers and floating point

Background: Excel stores IEEE-754 doubles. It displays at most 15 significant digits and applies
some extra rounding rules. Microsoft documents a few of them
(https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result),
but the exact rules must be measured.

## Near-zero snapping in addition and subtraction
Rule (hypothesis): when the result of a final `+`/`-` is within a few ULP of zero relative to the
operands, Excel returns exactly 0 (e.g. `=0.1+0.2-0.3` → 0), but not inside parentheses
(`=(0.1+0.2)-0.3`) and not in SUM.
Evidence: probes/precision-and-rounding.case.json A2, A3, A4, A15 (pending); suite probes-numeric `snap-addsub/*` (pending).
Status: UNVERIFIED (probe pending)

## ROUND uses decimal-correct half-away-from-zero
Rule (hypothesis): `ROUND(1.005,2)` = 1.01 and `ROUND(2.675,2)` = 2.68 in Excel, even though the
binary values are slightly below the half. IronCalc returns 1 for the first.
Evidence: probes/precision-and-rounding.case.json A6, A7 (pending).
Status: UNVERIFIED (probe pending)

## Number → text conversion uses 15 significant digits
Rule (hypothesis): `=CONCAT(0.1+0.2)` → "0.3" (IronCalc gives "0.30000000000000004").
Evidence: probes/precision-and-rounding.case.json A9 (pending); suite probes-numeric `number-to-text/*` (pending).
Status: UNVERIFIED (probe pending)

## Comparison tolerance
Rule (hypothesis): `=`, `<`, `>`, `<=`, `>=`, `<>` treat operands that differ by a few ULP (e.g.
`1+k*2^-52`, k 0..8) as equal, like the +/- snapping.
Evidence: suite probes-numeric `compare-tolerance/*` (pending).
Status: UNVERIFIED (probe pending)

## 15-digit literal parsing
Rule (hypothesis): numeric literals, `"text"+0` and `VALUE("text")` keep only 15 significant
digits; digits 16+ are dropped.
Evidence: suite probes-numeric `literal-15-digit/*` (pending).
Status: UNVERIFIED (probe pending)
