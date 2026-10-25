"""Independent loader for the r2-bratio verification: reads raw case formulas + oracle outputs, parses arguments
itself (formula literals cut to 15 significant digits, dyadic expressions evaluated in IEEE doubles)."""
import csv, json, math, re
from pathlib import Path

ROOT = Path("/home/user/rustsheet")
CASES = ROOT / "eval/suites/probes-numeric/cases"
T = ROOT / "docs/research/t2464"

TOK = re.compile(r"\s*(\d+\.?\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?|[-+*/^(),]|TRUE|FALSE)")


def lit(s):
    """Excel formula literal: digits 16+ are truncated (numbers.md "15-digit literal parsing")."""
    from decimal import Decimal, ROUND_DOWN
    d = Decimal(s)
    if d == 0:
        return 0.0
    e = d.adjusted()
    q = Decimal(1).scaleb(e - 14)
    return float(d.quantize(q, rounding=ROUND_DOWN))


class P:
    def __init__(self, s):
        self.t = []
        i = 0
        while i < len(s):
            m = TOK.match(s, i)
            if not m:
                raise ValueError(s[i:])
            self.t.append(m.group(1)); i = m.end()
        self.i = 0

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def eat(self, x=None):
        v = self.t[self.i]
        if x is not None and v != x:
            raise ValueError((v, x))
        self.i += 1
        return v

    def expr(self):  # + -
        v = self.term()
        while self.peek() in ("+", "-"):
            op = self.eat(); w = self.term(); v = v + w if op == "+" else v - w
        return v

    def term(self):
        v = self.unary()
        while self.peek() in ("*", "/"):
            op = self.eat(); w = self.unary(); v = v * w if op == "*" else v / w
        return v

    def unary(self):  # Excel: unary minus binds tighter than ^
        if self.peek() == "-":
            self.eat(); return -self.unary()
        return self.power()

    def power(self):
        v = self.atom()
        while self.peek() == "^":
            self.eat(); w = self.unary_atom(); v = math.pow(v, w)
        return v

    def unary_atom(self):
        if self.peek() == "-":
            self.eat(); return -self.atom()
        return self.atom()

    def atom(self):
        t = self.eat()
        if t == "(":
            v = self.expr(); self.eat(")"); return v
        if t == "TRUE":
            return True
        if t == "FALSE":
            return False
        return lit(t)


def parse_call(f, name="BETA.DIST"):
    m = re.fullmatch(r"=" + re.escape(name) + r"\((.*)\)", f)
    if not m:
        return None
    p = P(m.group(1)); args = [p.expr()]
    while p.peek() == ",":
        p.eat(","); args.append(p.expr())
    if p.peek() is not None:
        raise ValueError(f)
    return args


def oracle(case, outfile=None):
    if outfile is None:
        outfile = CASES / f"{case}.oracle.json"
    return json.load(open(outfile))["sheets"]


def beta_cases(case, outfile=None, setname=None, cell_refs=None):
    """BETA.DIST(x,a,b,TRUE) cells with 0<x<1 (A,B default) from a case; cell refs resolved from numeric cells."""
    d = json.load(open(CASES / f"{case}.case.json"))
    o = oracle(case, outfile)
    rows = []
    for sh in d["sheets"]:
        cells = sh["cells"]; osh = o.get(sh["name"], {})
        for ref, f in cells.items():
            if not (isinstance(f, str) and f.startswith("=BETA.DIST(")):
                continue
            ff = re.sub(r"\b([A-Z]{1,2}\d+)\b", lambda m: repr(cells[m.group(1)]) if m.group(1) in cells and not isinstance(cells[m.group(1)], str) else m.group(1), f)
            # cell-referenced values are stored doubles (not literals): substitute exactly via a marker
            args = None
            refs = re.findall(r"\b([A-Z]{1,2}\d+)\b", f)
            if refs:
                env = {r: cells[r] for r in refs}
                # parse with refs replaced by placeholder then substitute exact doubles
                s = f
                for r in refs:
                    s = re.sub(r"\b" + r + r"\b", "@" + r, s)
                args = _parse_env(s, env)
            else:
                args = parse_call(f)
            if args is None or len(args) != 4 or args[3] is not True:
                continue
            x, a, b = args[:3]
            if not 0 < x < 1:
                continue
            c = osh.get(ref)
            if not c or c.get("t") != "n":
                continue
            rows.append(dict(set=setname or case, case=case, cell=f"{sh['name']}!{ref}", a=a, b=b, x=x, y=1.0 - x,
                             side="lower", excel=float(c["v"])))
    return rows


def _parse_env(s, env):
    # supports '=BETA.DIST(@A1,a,b,TRUE)' and '=BETA.DIST(1-@A1,...)'
    m = re.fullmatch(r"=BETA\.DIST\((.*)\)", s)
    parts = _split(m.group(1))
    out = []
    for p in parts:
        mm = re.fullmatch(r"@([A-Z]+\d+)", p.strip())
        if mm:
            out.append(float(env[mm.group(1)])); continue
        if "@" in p:
            return None  # e.g. 1-A1: skip (not a cdf at a stored x)
        pp = P(p); out.append(pp.expr())
    return out


def _split(s):
    depth = 0; cur = ""; out = []
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def load_all():
    rows = []
    for c in ["t2464-w2f-xruns-1", "t2464-w2f-xruns-2", "t2464-w2f-absweep", "t2464-w2f-unit-shape"]:
        rows += beta_cases(c, setname="w2f")
    rows += beta_cases("t2464-r2-bratio-x29", setname="x29")
    # w4p1: the case lives in w4-fractions/probe (not in the suite dir)
    rows += _w4p1()
    # be: beta_exact.tsv cdf cells (from cells.tsv originals)
    for r in csv.DictReader(open(T / "beta_exact.tsv"), delimiter="\t"):
        if r["cum"] not in ("1", "True", "TRUE"):
            continue
        x, a, b = float(r["x"]), float(r["a"]), float(r["b"])
        rows.append(dict(set="be", case=r["case"], cell=r["cell"], a=a, b=b, x=x, y=1.0 - x, side="lower",
                         excel=float(r["excel"])))
    for r in csv.DictReader(open(T / "w5-triangulation" / "ix_evidence.tsv"), delimiter="\t"):
        if r["confidence"] not in ("direct", "derived", "closed") or not r["exact"]:
            continue
        rows.append(dict(set="ix", case=r["case"], cell=r["cell"], a=float(r["a"]), b=float(r["b"]), x=float(r["x"]),
                         y=float(r["y"]), side=r["side"], excel=float(r["excel_ix"]), fn=r["source_fn"]))
    return rows


def _w4p1():
    d = json.load(open(T / "w4-fractions/probe/w4p1.case.json"))
    o = json.load(open(T / "w4-fractions/probe/w4p1.out.json"))["sheets"]
    rows = []
    for sh in d["sheets"]:
        cells = sh["cells"]
        for ref, f in cells.items():
            if not (isinstance(f, str) and f.startswith("=BETA.DIST(A")):
                continue
            s = re.sub(r"\b(A\d+)\b", r"@\1", f)
            args = _parse_env(s, cells)
            if args is None or args[3] is not True:
                continue
            c = o.get(sh["name"], {}).get(ref)
            if not c or c.get("t") != "n":
                continue
            x, a, b = args[:3]
            rows.append(dict(set="w4p1", case="w4p1", cell=f"{sh['name']}!{ref}", a=a, b=b, x=x, y=1.0 - x,
                             side="lower", excel=float(c["v"])))
    return rows


def load_fresh():
    rows = []
    for c in ["shapes", "runs", "tiny", "tiny2"]:
        rows += beta_cases(f"t2464-r2-bpser-small-{c}", outfile=T / "r2-bpser-small" / f"t2464-r2-bpser-small-{c}.out.json",
                           setname=f"fresh-{c}")
    return rows
