# 0246 — Gate A reads ECMA.CEILING, ISO.CEILING, TRUNC, INT, MROUND, EVEN and ODD

- **Status:** accepted (2026-11-02, T-3674). Narrows SPEC-0029 "Excel behavior to match" for these seven functions,
  as decisions 0223 (ROUND family) and 0244 (CEILING and FLOOR family) do for theirs; decision 0244's last
  consequence left MROUND, TRUNC, ISO.CEILING and ECMA.CEILING to their own reading.
- **Context:**
  - Three of the seven are another function under a second name, each VERIFIED: TRUNC is ROUNDDOWN (numbers.md, "TRUNC
    is ROUNDDOWN", probes/trunc-decimal-stages, 56 cells), ECMA.CEILING is two-argument CEILING (numbers.md,
    "ECMA.CEILING", probes/ecma-ceiling), and ISO.CEILING is CEILING.PRECISE (every CEILING-family entry, the 16th-digit
    tie of T-2652/T-2687 and the subnormal ratio of T-2715, names it with CEILING.PRECISE and finds the same cells).
  - INT cuts to 15 significant digits (ties away from zero) below 1E+15 before it floors (numbers.md, "INT cuts to 15
    significant digits first", 68 cells). EVEN is `ceil(|x|/2)*2` and ODD `ceil(|x|)`, made odd, both signed (numbers.md,
    "ODD above 2^53 adds one in binary", 66 sweep cells); ODD's binary add is Excel's arithmetic, a unit in 2^53 at most.
  - MROUND takes its step count from the double quotient `|number/multiple|`, rounding up from a fraction of
    `0.5 - 5E-15` (numbers.md, "MROUND rounds the quotient up within 5E-15 of a half, and multiplies in binary", 213
    cells). The tolerance is defined on that double: a decimal half such as `MROUND(12.9,0.2)` (64.5) has a double
    quotient that can sit an ulp off the half, and above a quotient of about 32 an ulp is more than 5E-15, so the exact
    ratio and the double quotient can choose different steps. Read on the exact ratio, such a point would put Excel a
    whole multiple (up to 1/k relative, k the step count) off the math, a false decision 0188 R3 mark, as decision 0244
    says of the CEILING family's cut.
- **Decision:**
  - TRUNC, ECMA.CEILING and ISO.CEILING share the definitions of ROUNDDOWN (decision 0223), CEILING and CEILING.PRECISE
    (decision 0244), draws included; each table holds the two-argument form (the optional-argument forms are eval cells).
  - INT's math is the floor of the written decimal (the 15-digit cut of a table literal is its written decimal; from
    1E+15 up the double is already whole). EVEN's and ODD's math is their rule at the double, exactly.
  - MROUND's step count is Excel's rule on the double quotient; the math is that count times |multiple| exactly, with
    the sign of `number`. A zero multiple gives 0; opposite signs are a domain error (`#NUM!`; both checks evidenced
    by accuracy/MROUND/s1 itself, numbers.md's MROUND entry); a double quotient that
    overflows has no math value (no Excel cell evidences it). The draws avoid the cutoff `0.5 - 5E-15` itself.
  - Definitions: `tools/src/rustsheet_tools/accuracy/defs/integer.py` (INT, EVEN, ODD, MROUND), `defs/rounding.py`
    (TRUNC) and `defs/ceiling_floor.py` (ECMA.CEILING, ISO.CEILING).
- **Consequences:**
  - The T-3674 Outcome lists, from the filled tables, every failing point with its owner task and whether Excel is
    more than G = 1e-4 off this math anywhere (an R3 candidate needs an owner ruling).
  - A later change of Excel's MROUND rule (another tolerance, a decimal quotient) changes this definition, not the
    envelope: the evidence for the double quotient is T-2201's 213 cells.
