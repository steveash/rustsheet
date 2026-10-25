"""Independent loader for BETA.DIST pdf cells with a > 1, b > 1 (verifier of r2-density-large).

Sources parsed from scratch (formula text + oracle values), not from any worker's tables:
  old:  cells.tsv BETA.DIST rows with cumulative FALSE (A=0, B=1 or omitted)
  w2:   eval/.../t2464-w2f-*.case.json + .oracle.json
  pr1:  w1-density/probe/probe1.case.json + probe1.out.json
  p1:   eval/.../t2464-r2-density-large-p1.case.json + r2-density-large/probe_p1.out.json
"""
from __future__ import annotations
import ast, csv, glob, json, re, pickle
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
CELLREF = re.compile(r"^\$?([A-Z]+)\$?(\d+)$")


def _split_args(s):
    """Top-level comma split of a call's argument text (parenthesis depth 0), each argument stripped."""
    parts, depth, start = [], 0, 0
    for pos, ch in enumerate(s):
        depth += (ch == "(") - (ch == ")")
        if ch == "," and depth == 0:
            parts.append(s[start:pos])
            start = pos + 1
    parts.append(s[start:])
    return [a.strip() for a in parts]


def _ev(node, cells):
    if isinstance(node, ast.Expression):
        return _ev(node.body, cells)
    if isinstance(node, ast.Constant):
        return Fraction(node.value) if not isinstance(node.value, float) else Fraction(node.value)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return -_ev(node.operand, cells)
    if isinstance(node, ast.BinOp):
        l, r = _ev(node.left, cells), _ev(node.right, cells)
        if isinstance(node.op, ast.Add): return l + r
        if isinstance(node.op, ast.Sub): return l - r
        if isinstance(node.op, ast.Mult): return l * r
        if isinstance(node.op, ast.Div): return l / r
        if isinstance(node.op, ast.Pow): return l ** int(r)
    if isinstance(node, ast.Name):
        if node.id in ("TRUE", "FALSE"):
            return node.id == "TRUE"
        v = cells[node.id]
        return Fraction(v)
    raise ValueError(ast.dump(node))


def arg_value(txt, cells):
    t = txt.strip()
    if t.upper() in ("TRUE", "FALSE"):
        return t.upper() == "TRUE"
    # numeric literal: parse as Python float (= correctly rounded decimal, as Excel's parser intends)
    try:
        return float(t)
    except ValueError:
        pass
    py = t.replace("^", "**")
    v = _ev(ast.parse(py, mode="eval"), cells)
    # every sub-term in these encoders (m*2^-k) is exact in double; the single final add is rounded once
    return float(v)


def parse_formula(f, cells):
    m = re.match(r"^=BETA\.DIST\((.*)\)$", f.strip(), re.I)
    if not m:
        return None
    args = [arg_value(a, cells) for a in _split_args(m.group(1))]
    return args


def _keep(args):
    if len(args) not in (4, 6):
        return None
    x, a, b, cum = args[:4]
    if cum is not False:
        return None
    if len(args) == 6 and (args[4] != 0 or args[5] != 1):
        return None
    if not (a > 1 and b > 1 and 0 < x < 1):
        return None
    return float(x), float(a), float(b)


def _from_case(src, case, values):
    rows = []
    for sh in case["sheets"]:
        cells = {k: v for k, v in sh["cells"].items() if not (isinstance(v, str) and v.startswith("="))}
        vals = values.get(sh["name"], {})
        for addr, f in sh["cells"].items():
            if not (isinstance(f, str) and f.upper().startswith("=BETA.DIST(")):
                continue
            k = _keep(parse_formula(f, cells))
            if k is None:
                continue
            c = vals.get(addr)
            if not c or c.get("t") != "n":
                continue
            rows.append(dict(src=src, cell=f"{sh['name']}!{addr}", x=k[0], a=k[1], b=k[2], excel=float(c["v"])))
    return rows


def load():
    cache = HERE / "vrows.pkl"
    if cache.exists():
        return pickle.loads(cache.read_bytes())
    rows = []
    for r in csv.DictReader(open(T / "cells.tsv"), delimiter="\t"):
        if r["fn"] != "BETA.DIST":
            continue
        args = [ast.literal_eval(a) for a in r["args"].split("|")]
        args = [a if isinstance(a, bool) else float(a) for a in args]
        k = _keep(args)
        if k is None:
            continue
        rows.append(dict(src="old:" + r["case"], cell=r["cell"], x=k[0], a=k[1], b=k[2], excel=float(ast.literal_eval(r["excel"]))))
    for f in sorted(glob.glob(str(CASES / "t2464-w2f-*.case.json"))):
        o = f.replace(".case.json", ".oracle.json")
        rows += _from_case("w2:" + Path(f).name, json.load(open(f)), json.load(open(o))["sheets"])
    rows += _from_case("pr1", json.load(open(T / "w1-density/probe/probe1.case.json")),
                       json.load(open(T / "w1-density/probe/probe1.out.json"))["sheets"])
    rows += _from_case("p1", json.load(open(CASES / "t2464-r2-density-large-p1.case.json")),
                       json.load(open(T / "r2-density-large/probe_p1.out.json"))["sheets"])
    cache.write_bytes(pickle.dumps(rows))
    return rows


if __name__ == "__main__":
    from collections import Counter
    rows = load()
    print(len(rows), Counter(r["src"].split(":")[0] for r in rows))
    print(Counter(r["src"] for r in rows).most_common(40))
    keys = Counter((r["x"], r["a"], r["b"]) for r in rows)
    d = sum(v - 1 for v in keys.values() if v > 1)
    print("duplicate (x,a,b) extra copies:", d, "unique:", len(keys))
    # conflicting duplicates
    by = {}
    for r in rows:
        by.setdefault((r["x"], r["a"], r["b"]), set()).add(r["excel"])
    print("dup keys with differing excel values:", sum(1 for v in by.values() if len(v) > 1))
