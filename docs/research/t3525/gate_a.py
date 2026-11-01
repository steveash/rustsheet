"""T-3525: the Gate A envelope (decision 0188 R2) of today's engine on the fresh sweeps.

A cell is inside the envelope when every part of the engine's IM* text is within 1e-9 relative of the
50-digit mpmath value, or when the engine's result equals Excel's (same 15-digit text, or the same error).
An error against a number, either way, is outside whatever the math says (SPEC-0029 `ErrorMismatch`).
Writes docs/research/t3525/gate_a.tsv (one row per cell: the 50-digit parts, the engine's and Excel's
result, the worst relative error and the verdict) and prints the counts and every cell outside.

Run from tools/: uv run python ../docs/research/t3525/gate_a.py
"""

from __future__ import annotations

import re
from pathlib import Path

import mpmath

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import im_hyperbolic, registry
from rustsheet_tools.fitlab.score import ExcelError, score
from rustsheet_tools.fitlab.sweep import load_sweep, parse_complex_text

OUT = Path(__file__).resolve().parent / "gate_a.tsv"
FUNCTION = re.compile(r"^=(IM\w+)\(")


def main() -> None:
    spec = registry()["im-hyperbolic-fresh"]
    sweep = load_sweep(spec.cases, spec.pattern, None, True, complex_text=True)
    engine = fit.engine_candidate(sorted({c.case for c in sweep.cells}))
    rows = ["case\tcell\tformula\tmpmath_re\tmpmath_im\tengine\texcel\tworst_rel\tverdict"]
    counts: dict[str, dict[str, int]] = {}
    outside: list[str] = []
    for cell in sweep.cells:
        name = FUNCTION.match(cell.formula)
        arg = cell.args[0]
        if name is None or not isinstance(arg, str):
            continue
        x, y = parse_complex_text(arg)
        with mpmath.workdps(50):
            z = im_hyperbolic._KERNELS[name[1]](mpmath.mpc(x, y))
            exact = (mpmath.nstr(z.real, 20), mpmath.nstr(z.imag, 20))
            try:
                text = str(engine.evaluate(cell))
            except ExcelError as e:
                text = e.code
            if text.startswith("#") or cell.error is not None:
                rel = "-"
                inside = "excel" if text == cell.error else ""
            else:
                re_part, im_part = parse_complex_text(text)
                worst = max(
                    abs(mpmath.mpf(a) - b) / abs(b) if b != 0 else abs(mpmath.mpf(a))
                    for a, b in ((re_part, z.real), (im_part, z.imag))
                )
                rel = f"{float(worst):.3g}"
                inside = (
                    "math"
                    if worst <= mpmath.mpf("1e-9")
                    else ("excel" if score([cell], engine).exact else "")
                )
        excel = cell.error or f"{cell.expected!r}{cell.expected_im:+}i"
        verdict = inside or "OUTSIDE"
        case = cell.case.rsplit("/", 1)[-1].removesuffix(".case.json")
        rows.append(
            f"{case}\t{cell.key}\t{cell.formula}\t{exact[0]}\t{exact[1]}\t{text}\t{excel}\t{rel}\t{verdict}"
        )
        per_case = counts.setdefault(case, {})
        per_case[verdict] = per_case.get(verdict, 0) + 1
        if not inside:
            outside.append(f"  {case} {cell.key} {cell.formula}: engine {text}, Excel {excel}")
    OUT.write_text("\n".join(rows) + "\n", encoding="utf-8")
    for case, c in counts.items():
        print(
            f"{case}: {sum(c.values())} cells, within 1e-9 of mpmath {c.get('math', 0)}, "
            f"else equal to Excel {c.get('excel', 0)}, outside {c.get('OUTSIDE', 0)}"
        )
    print("outside:", *outside, sep="\n")


if __name__ == "__main__":
    main()
