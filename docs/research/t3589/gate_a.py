"""T-3589: the Gate A envelope (decision 0188 R2) of the frozen model on its fresh sweeps.

A cell is inside the envelope when every part of the engine's IM* text is within 1e-9 relative of the
50-digit mpmath value, or when the engine's text equals Excel's. Covers all six functions of
docs/research/t3589/gen_sweep.py's two cases (IMSECH and IMCSCH included, which the
`im-hyperbolic-t3589-fresh` fit leaves out). Prints the counts per function and every cell outside.

Run from tools/: uv run python ../docs/research/t3589/gate_a.py
"""

from __future__ import annotations

import re

import mpmath

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.score import ExcelError
from rustsheet_tools.fitlab.sweep import load_sweep, parse_complex_text

CASES = [
    "probes-numeric/t3589-im-hyperbolic-small-x-sweep",
    "probes-numeric/t3589-imsin-imcos-large-imag-sweep",
]
PATTERN = r"^=IM(?:SINH|COSH|SECH|CSCH|SIN|COS)\((.*)\)$"
FUNCTION = re.compile(r"^=(IM\w+)\(")
KERNELS = {
    "IMSINH": mpmath.sinh,
    "IMCOSH": mpmath.cosh,
    "IMSECH": mpmath.sech,
    "IMCSCH": mpmath.csch,
    "IMSIN": mpmath.sin,
    "IMCOS": mpmath.cos,
}


def main() -> None:
    sweep = load_sweep(CASES, PATTERN, None, True, complex_text=True)
    engine = fit.engine_candidate(sorted({c.case for c in sweep.cells}))
    counts: dict[str, dict[str, int]] = {}
    outside: list[str] = []
    for cell in sweep.cells:
        name = FUNCTION.match(cell.formula)
        arg = cell.args[0]
        if name is None or not isinstance(arg, str):
            continue
        x, y = parse_complex_text(arg)
        try:
            text = str(engine.evaluate(cell))
        except ExcelError as e:
            text = e.code
        excel = cell.error or f"{cell.expected!r}{cell.expected_im:+}i"
        if text.startswith("#") or cell.error is not None:
            verdict = "excel" if text == cell.error else "OUTSIDE"
        else:
            with mpmath.workdps(50):
                z = KERNELS[name[1]](mpmath.mpc(x, y))
                re_part, im_part = parse_complex_text(text)
                worst = max(
                    abs(mpmath.mpf(a) - b) / abs(b) if b != 0 else abs(mpmath.mpf(a))
                    for a, b in ((re_part, z.real), (im_part, z.imag))
                )
            verdict = "math" if worst <= mpmath.mpf("1e-9") else "OUTSIDE"
        per = counts.setdefault(name[1], {})
        per[verdict] = per.get(verdict, 0) + 1
        if verdict == "OUTSIDE":
            outside.append(f"  {cell.key} {cell.formula}: engine {text}, Excel {excel}")
    for function, c in sorted(counts.items()):
        print(
            f"{function}: {sum(c.values())} cells, within 1e-9 of mpmath {c.get('math', 0)}, "
            f"else equal to Excel {c.get('excel', 0)}, outside {c.get('OUTSIDE', 0)}"
        )
    print("outside:", *outside, sep="\n")


if __name__ == "__main__":
    main()
