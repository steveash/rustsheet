# Fit lab

The fit lab (`tools/src/rustsheet_tools/fitlab/`, T-1523) scores candidate implementations of a
numeric kernel against an oracle sweep and ranks them. Decision 0056 makes every last-digit fit
such a comparison: the fit task records each candidate's bit-exact and within-4-ulp counts,
including the rejected ones. Decision 0026 still applies: no fit without a sweep.

## Run it

From `tools/`:

```
uv run rustsheet-fitlab --list                 # shipped fits
uv run rustsheet-fitlab erfc                   # ERFC + ERFC.PRECISE, engine vs libm (530 cells)
uv run rustsheet-fitlab erfc-tail --top 10     # the T-1187 grid, about 5,800 candidates (~10 s)
uv run rustsheet-fitlab erfc-tail --only 'bu_even' --worst 10 --no-engine
uv run rustsheet-fitlab --case probes-numeric/t1040-erfc \
    --pattern '^=ERFC\((.*)\)$' --candidate math:erfc --candidate mymod:my_erfc   # ad hoc
```

The output has three parts:

- A ranked table: bit-exact, `<=4 ulp` (exact included), `<=1e-9` relative, errors and the worst
  ulp distance per candidate. Ranking is by exact count, then the 4-ulp count, then the 1e-9 count,
  then the worst distance. The engine's row is always shown.
- A family table, when candidates are grouped. It gives each family's best member, the union of
  the cells its members get exactly, and the cumulative union.
- The worst cells of the engine and of the best three candidates.

The JSON (`eval/runs/fitlab/<fit>.json`, git-ignored, or `--json PATH`) holds every candidate's
counts and worst cells, the per-family unions, and the cells the reader could not parse.

The verdicts are the harness's (`crates/rustsheet-eval/src/value.rs`). `score.ulp_distance` is a
port of `ulp_distance`, so 0.0 and -0.0 are 0 apart. A candidate that raises, or returns something
other than a number, misses that cell and is counted under `err`. It does not stop the run.

## Inputs

- **Sweep:** an oracle-filled `.case.json` plus its `.oracle.json` sidecar, found through
  `rustsheet-eval cases` (`snapshot.suite_cases`). No xlsx is parsed. Only cells whose Excel value
  is a number are scored. The others are counted as non-numeric.
- **Cell selection:** a regex over the formula text. Group 1 is the argument list, for example
  `^=GAMMA\.DIST\((.*)\)$`. The arguments are split at top-level commas. The reader handles
  numeric literals, TRUE/FALSE, string literals, and references (`A1`, `$Z$1`, `Data!B2`,
  `D1:D10`, which gives a row-major list). A reference to a constant cell gives the case's value.
  A reference to a formula cell gives Excel's value from the sidecar. A cell with any other
  argument, such as a nested expression, is listed as unparsed.
- **The engine:** a pseudo-candidate read from `rustsheet-eval show --all <case>`
  (`snapshot.show_cells`). It needs the release harness binary, which `snapshot.eval_binary`
  builds on first use. `--no-engine` skips it.

## Add a candidate or a fit

A candidate is `score.candidate(name, fn, family)`. `fn` receives the cell's parsed arguments,
in order, and returns a float. A fit is a `fit.Fit`:

```python
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import candidate

def my_gamma(x: float) -> float: ...

FITS = [
    Fit(
        name="gamma",
        description="GAMMA over the T-1042 sweeps",
        cases=["probes-numeric/t1042-gamma-int-1-90", "probes-numeric/t1042-gamma-half-int"],
        pattern=r"^=GAMMA\((.*)\)$",
        candidates=lambda: [candidate("math.gamma", math.gamma), candidate("lanczos g=7", my_gamma)],
        where=lambda cell: cell.args[0] > 0,  # optional: a sub-domain
    )
]
```

To ship a fit:

1. Put it in `fitlab/kernels/<kernel>.py`.
2. Add the module to `MODULES` in `fitlab/kernels/__init__.py`.
3. Record the table in the fit task's Outcome.

Group the variants of one idea (depths, tolerances, operation orders) under one `family`, so the
family table shows how far the whole idea gets. `candidates` is a function, so a large family is
only built when the fit runs. `--only REGEX` narrows a run to some candidates.

## Exact arithmetic (`fitlab/exact.py`)

Candidates that model Excel's rounding use `Fraction`s:

- `round_to_bits(x, p, min_exp=None)`: round to nearest with a p-bit significand, ties to even.
  Use p = 53 for doubles and p = 64 for the x87 extended significand. `min_exp=-1074` gives double
  subnormals. `to_double` rounds correctly to a double.
- The x87 model (numbers.md "Division rounds twice"): `double_round(x)` rounds the exact value to
  64 bits, then to a double. `X87.add/sub/mul/div/sqrt/exp/ln/pow` apply this to one operation on
  doubles, as one Excel operator does: `X87.div(1, 2 - 2**-52)` is 0.5, where the correctly
  rounded result is 0.5000000000000001.
- `Ext` holds a value at 64 bits across a chain of operations, as in an x87 register. Only
  `.to_double()` rounds to a double. Use it when testing whether a compiled kernel keeps its
  intermediates in extended precision (T-1524).
- `sqrt_cr`, `exp_cr`, `ln_cr` and `pow_cr(x, y, p=53)` are correctly rounded to p bits:
  - `sqrt_cr` uses an exact integer square root.
  - `exp_cr` and `ln_cr` use `decimal`, whose exp and ln are correctly rounded at the working
    precision. The precision is raised from 40 digits up to 2,400 until the error bound no longer
    straddles a rounding boundary (Ziv's strategy).
  - `pow_cr` returns exact powers (integer exponents, or results such as 4^0.5) exactly. A result
    that would need more than 2,400 digits raises `ArithmeticError`.

  `X87.exp`, `X87.ln` and `X87.pow` are idealized. They round correctly to 64 bits, which the
  x87 microcode instructions (`f2xm1`, `fyl2x`) do not.

## Shipped fits

| fit | cells | candidates |
|---|---|---|
| `erfc` | 530 (ERFC and ERFC.PRECISE, x in -6..27) | engine, `math.erfc`, `1 - math.erf` |
| `erfc-tail` | 205 (x >= 1, Excel non-zero) | engine, libm, and the T-1187 continued-fraction, Lentz and Numerical Recipes families (5,794 in all) |

The `erfc-tail` fit replaces `docs/research/t1187-erfc-candidates.py`. It reproduces that
script's counts: best 57, 58, 38 and 4, with a union of 174/205.
