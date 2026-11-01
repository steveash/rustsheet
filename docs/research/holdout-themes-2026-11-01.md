You are working on RustSheet (steveash/rustsheet). A private holdout evaluation, run daily against
`main`, scores RustSheet on Excel-computed cases that RustSheet's agents never see. This note gives you
its **themes only**. It deliberately leaves out formulas, inputs and expected values, and you must not
try to recover or guess them: the holdout only stays useful if fixes come from Excel evidence that
RustSheet gathers itself.

## How to use this note

- Treat each theme as a lead for **your own investigation**. File tasks the usual way (`docs/tasks/`,
  one lane, size S/M), and back every behaviour change with fresh oracle evidence through the
  `excel-probe` skill and RustSheet's own suites. Do not cite this note as evidence for any Excel behaviour.
- Generalize. Each theme names a *class* of input (a function family plus a kind of argument), not a
  failing case. A good task covers the whole class broadly across the family (the whole argument
  domain, every coercion path, every sibling function), not one point.
- Don't tune for this note. Don't add test cases that copy its wording, and don't special-case anything.
  If your probes show RustSheet already matches Excel for a theme, record that and move on.
- Leave knowledge behind as usual: quirks go in `docs/excel-semantics/`.

## Overall picture

About 95% of holdout cases pass. Lookup, logic, math, text and date are at 99%+ of cells; criteria,
engineering and financial are at 97-98%; statistical is at about 85%. Most statistical misses are not
*visibly* wrong: they agree to 15 significant digits but differ in the last bits. So the themes
fall into two groups: (A) visibly wrong answers (wrong value, wrong error, or a value where Excel
gives an error or the reverse), and (B) bit-level precision.

## A. Visibly wrong answers (highest priority)

1. **FORECAST.ETS family is unimplemented.** FORECAST.ETS, .CONFINT, .SEASONALITY and .STAT return
   an engine-internal not-implemented error where Excel gives values or errors. This is the largest
   single block of failures. (Existing task files mention ETS probes. Check whether an implementation
   task exists and what blocks it before filing a new one.)

2. **Complex-number functions (IM\*) at extreme magnitudes and on malformed input.** In the
   trigonometric IM functions (IMSIN, IMCOS, IMTAN, IMSEC, IMCSC, IMCOT and the hyperbolic ones),
   RustSheet returns a complex string where Excel returns #NUM!, mostly for very large or very small
   components. Separately, IMLOG10, IMLOG2, IMSQRT and IMSECH produce text that differs from Excel's
   in some regions. Look at how Excel detects overflow or loss of meaning in intermediate results,
   how it validates and parses complex strings, and how it formats the result.

3. **Text-to-number and text-to-date parsing is too strict in places and too lenient in others.**
   - DATEVALUE returns #VALUE! for date texts that Excel accepts. Survey the en-US date text formats Excel accepts.
   - NUMBERVALUE handling of separators and spaces disagrees in both directions.
   - ARABIC rejects some inputs Excel accepts.
   - TEXT returns a string for some format/value combinations where Excel gives #VALUE!.
   - XIRR and XNPV coerce their date and value arguments differently from Excel, including which error comes out.

   These may share a common gap in the shared text-coercion helpers, so look for one root cause before
   patching each function.

4. **Database functions (D\*) share failures.** All twelve D-functions fail on the same kinds of input:
   criteria-range grammar (wildcards, comparison operators, an empty or unmatched criteria block), how
   the `field` argument is resolved (text versus position, out-of-range values), and error precedence.
   In some cases Excel gives #VALUE! and RustSheet gives #N/A, or a number versus #N/A. Because the
   failures are identical across the family, they almost certainly come from the shared D-function
   front end, not from each function.

5. **Engineering base conversion (BIN2\*, and probably the OCT and HEX families).** At input-length and
   sign boundaries, and for text-number or coerced arguments, Excel gives #NUM! where RustSheet returns
   a value. Review argument validation (digit count, the negative two's-complement range, places) across
   all twelve base-conversion functions together.

6. **Odd-period bond functions (ODDFYIELD, ODDFPRICE).** RustSheet returns values where Excel gives
   #NUM!, and returns visibly different numbers across broad combinations of rate, yield, frequency and
   basis. This suggests differences in validating the date order and domain, as well as in the
   solver/formula itself. The ODDL\* pair is mostly fine; compare them.

7. **Regression family (LINEST, LOGEST, and to a lesser degree TREND/GROWTH).** Some output cells differ
   visibly, not only in the last bits, especially in the extended statistics block and in degenerate or
   collinear designs. Check how Excel handles rank deficiency and its "const"/"stats" combinations.

8. **Smaller clusters:**
   - PERCENTOF: which error is returned for error and coerced arguments.
   - FILTER: a few results have the wrong type.
   - MAKEARRAY: errors for arity and boundary sizes.
   - INDIRECT: reference text with spaces or a reversed range.
   - PROPER and LOWER: some non-ASCII Unicode case mappings differ from Excel's.
   - DEVSQ: visibly off on near-cancelling data. Excel probably uses a different summation or centering order.

## B. Bit-level precision (lower priority, but large volume)

Most of the remaining misses are within 15 significant digits but more than a few ulp away, and they
cluster in:
- **inverse distributions**: F.INV / FINV / F.INV.RT, BETA.INV, GAMMA.INV, CHISQ.INV(.RT) / CHIINV,
  T.INV / T.INV.2T / TINV;
- **discrete and gamma distributions**: GAMMA.DIST, NEGBINOMDIST, HYPGEOM.DIST;
- **tests**: T.TEST, CONFIDENCE.T;
- **regression**: LINEST, LOGEST, GROWTH, TREND;
- **financial**: CUMPRINC.

This points to differences in algorithm, not one-off bugs: the series or continued-fraction
evaluation, the inversion method and its stopping rule, and the accumulation order. A small number of
tasks aimed at the shared numerical kernels (incomplete beta and gamma, the inverse-CDF solver, and
the least-squares solver) would probably move many functions at once. Characterize Excel across each
function's whole domain with your own probes, not at single points.

## What is already strong (don't churn it)

Lookup and dynamic arrays (XLOOKUP, XMATCH, the stack/wrap/take/drop family), logic and information
functions, most math, date arithmetic, and the conditional aggregates (*IF/*IFS) all score 98-99.9%.
Be careful with refactors that touch them.
