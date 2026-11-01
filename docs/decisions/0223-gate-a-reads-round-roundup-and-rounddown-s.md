# 0223 — Gate A reads ROUND, ROUNDUP and ROUNDDOWN's math at the written decimal

- **Status:** accepted (2026-11-01, T-3555). Narrows SPEC-0029 "Excel behavior to match" for these three functions.
- **Context:**
  - SPEC-0029 evaluates a Gate A point's math "at the double nearest the written decimal". For a decimal rounding
    function that makes Excel's documented rule look like an error. The double nearest 2.675 is
    2.67499999999999982236431605997495353221893310546875, which rounds to 2.67. Excel's `ROUND(2.675,2)` is 2.68, a
    relative gap of 3.7e-3, more than decision 0188 R3's G = 1e-4.
  - Excel's rule is VERIFIED (numbers.md, "ROUND family rounds in two decimal stages": probes/round-family A1–A53,
    probes/precision-and-rounding A6–A8). It cuts the binary value to 15 significant digits, then rounds that decimal:
    half away from zero for ROUND, away from zero for ROUNDUP, toward zero for ROUNDDOWN.
  - A table literal has at most 15 significant digits, so its double cut to 15 digits is the written decimal again.
  - Excel is not inaccurate here. Its rule is decimal by design, so an R3 mark (which says Excel is far from the
    math) would be false.
- **Decision:**
  - The math of ROUND, ROUNDUP and ROUNDDOWN is the argument's 15-digit decimal rounded at `num_digits` by the
    function's rule. A result beyond the double range is a math domain error (`ROUNDUP(1.5,-400)` is `#NUM!`).
  - Two cases have no math value. These are never drawn, and `rustsheet-accuracy math` gives such eval cells none:
    - an exact tie at the 16th digit, which only a computed argument can carry, and which Excel breaks toward zero
      for ROUND and away from zero for ROUNDUP and ROUNDDOWN;
    - a `num_digits` beyond ±400, which Excel narrows first (numbers.md "ROUND family: huge num_digits wrap", and
      T-3627's open observation at -43,376).
  - Definition: `tools/src/rustsheet_tools/accuracy/defs/rounding.py`.
- **Consequences:**
  - On T-3555's tables (360 points, excel-graph 2026-11-01), Excel is more than G off this math at 0 points. Every
    point is in the envelope.
  - ROUNDUP's binary add of the unit (numbers.md) leaves Excel and the engine 1 ulp from the decimal math at some
    points. That is far inside ε, so it needs no rule here.
  - Other decimal rounding functions (MROUND, TRUNC, CEILING, FLOOR, FIXED, DOLLAR) need their own reading when they
    get tables. This record covers only the three.
