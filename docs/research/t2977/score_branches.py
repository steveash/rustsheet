"""T-2977: score the engine's incomplete beta K(a, b, x, y) per bratio branch, after the seams were deleted.

Rows: r2-bratio's `score_lib.load` sets (be, w2f, w4p1, ix, x29) and r2-bpser-small's probe
t2464-r2-bpser-small-shapes. Each row runs through the engine's `bratio::kernel_tails` (closed forms, the ported
branches, else today's kernel) with the T-2464 research harness op `BRATIO`, which also reports the branch bratio
dispatches to and whether a ported form computed the point. The harness was deleted in the same task, so rerunning
this needs it back from git history (`git show <T-2977 parent>:crates/rustsheet-engine/src/functions/statistical/
t2464_harness.rs`, its `mod` line, and `cargo test --release -p rustsheet_engine --lib t2464_kernels --no-run`).

Usage (repo root; data rebuilt per docs/research/t2506/RESULTS.md "Data"):
  T2464_HARNESS=<engine test binary> bindings/python/.venv/bin/python docs/research/t2977/score_branches.py
Scores: bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15).
"""
import os
import struct
import subprocess
import sys
import tempfile
from collections import defaultdict
from dataclasses import replace
from pathlib import Path

T2464 = Path(__file__).resolve().parent.parent / "t2464"
sys.path.insert(0, str(T2464 / "r2-bratio"))
sys.path.insert(0, str(T2464 / "r2-bpser-small"))
import bratio_excel as bx  # noqa: E402
import lib as bsl  # noqa: E402
from score_lib import load, tally  # noqa: E402

BRANCHES = ["zero", "one", "tiny", "fpser", "apser", "w_bpser", "w1_bpser", "bup_bgrat", "bgrat", "bpser_large",
            "l140_bup_bpser", "l140_bup_bgrat", "l140_bup_bup_bgrat", "bfrac", "basym"]
HARNESS = os.environ["T2464_HARNESS"]


def hx(v: float) -> str:
    return f"{struct.unpack('<Q', struct.pack('<d', float(v)))[0]:016x}"


def fl(h: str) -> float:
    return struct.unpack("<d", struct.pack("<Q", int(h, 16)))[0]


def engine(rows):
    with tempfile.TemporaryDirectory() as d:
        inp, out = os.path.join(d, "in"), os.path.join(d, "out")
        with open(inp, "w") as f:
            for r in rows:
                f.write(f"BRATIO {hx(r['a'])} {hx(r['b'])} {hx(r['x'])} {hx(r['y'])}\n")
        env = dict(os.environ, T2464_IN=inp, T2464_OUT=out)
        subprocess.run([HARNESS, "--ignored", "--exact", "functions::statistical::t2464_harness::t2464_kernels",
                        "--quiet"], env=env, check=True, capture_output=True)
        res = []
        for line in open(out):
            w, w1, br, ported = line.split()
            res.append((fl(w), fl(w1), BRANCHES[int(br)] if int(br) < len(BRANCHES) else "none", ported == "1"))
        return res


def b0_class(r) -> str:
    """r2-bpser-small's sub-branch of the bpser call: by b0 = max(a, b) (`lib.sub_branch`)."""
    b = max(r["a"], r["b"])
    return "b0<=1" if b <= 1 else ("1<b0<8" if b < 8 else "b0>=8")


def closed(r) -> bool:
    """Whether Excel's closed forms take the point (`incomplete_beta::closed_form_tails`)."""
    if not (r["a"] == 1 or r["b"] == 1):
        return False
    a0, b0, y0 = (r["b"], r["a"], r["x"]) if r["x"] > 0.5 else (r["a"], r["b"], r["y"])
    return b0 == 1 or (a0 == 1 and y0 <= 0.9)


def fmt(c) -> str:
    n, e, u, v = tally((m, x) for m, x, _ in c)
    _, ce, cu, cv = tally((cr, x) for _, x, cr in c)
    return f"{n} | {e}/{u}/{v} | {ce}/{cu}/{cv}"


def table(title, groups):
    print(f"\n## {title}\n| group | n | engine ex/u4/w15 | CR ex/u4/w15 |\n|---|---|---|---|")
    for g in sorted(groups):
        print(f"| {g} | {fmt(groups[g])} |")


def main() -> None:
    rows = load(sets=("be", "w2f", "w4p1", "ix", "x29"))
    res = engine(rows)
    by_branch, by_set = defaultdict(list), defaultdict(list)
    for r, (w, w1, br, ported) in zip(rows, res):
        cell = (w if r["side"] == "lower" else w1, r["excel"], r["cr"])
        if closed(r):
            br, ported = "closed", True
        by_branch[br].append(cell)
        by_branch[f"{br} ({'ported' if ported else 'today'})"].append(cell)
        by_set[f"{r['set']} {br}"].append(cell)
        by_branch["ALL"].append(cell)
    table("r2-bratio sets by branch", by_branch)
    table("r2-bratio sets by set and branch", by_set)

    # Both L forms of bpser's ln B on the bpser rows: r2-bratio's model with w3_lg = toms (BEST) and = gl (Excel's
    # GAMMALN, the engine's GL53), next to the engine.
    bp = [(r, e) for r, e in zip(rows, res) if e[2] in ("w_bpser", "w1_bpser", "bpser_large") and not closed(r)]
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r, _ in bp]
    models = {"toms": bx.run(pts, bx.BEST), "gl": bx.run(pts, replace(bx.BEST, w3_lg="gl"))}
    print("\n## bpser rows: engine and r2-bratio's model with each L form (ex/u4/w15)\n"
          "| branch | n | engine | model w3_lg=toms | model w3_lg=gl |\n|---|---|---|---|---|")
    for br in ("w_bpser", "w1_bpser", "bpser_large"):
        idx = [i for i, (_, e) in enumerate(bp) if e[2] == br]
        side = [bp[i][0]["side"] == "lower" for i in idx]
        cols = [tally(((bp[i][1][0] if lo else bp[i][1][1]), bp[i][0]["excel"]) for i, lo in zip(idx, side))]
        for m in ("toms", "gl"):
            cols.append(tally(((models[m][i][0] if lo else models[m][i][1]), bp[i][0]["excel"])
                              for i, lo in zip(idx, side)))
        print(f"| {br} | {len(idx)} | " + " | ".join(f"{c[1]}/{c[2]}/{c[3]}" for c in cols) + " |")

    # bpser criterion: t2464-r2-bpser-small-shapes by bpser branch and b0.
    shapes = bsl.load_probe(("shapes",))
    sres = engine(shapes)
    groups = defaultdict(list)
    for r, (w, _w1, br, ported) in zip(shapes, sres):
        cell = (w, r["excel"], r["cr"])
        if closed(r):
            br = "closed"
        if br in ("w_bpser", "w1_bpser"):
            groups[f"{br} {b0_class(r)}"].append(cell)
            groups[f"bpser {b0_class(r)}"].append(cell)
        else:
            groups[f"{br}{'' if ported else ' (today)'}"].append(cell)
    table("t2464-r2-bpser-small-shapes", groups)

    # x29 criterion: the cells in [0.29, 0.3) where the 0.29 and 0.3 dispatches give different doubles.
    x29 = [r for r in rows if r["set"] == "x29" and r["x"] < 0.3]
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r in x29]
    v29 = bx.run(pts, replace(bx.BEST, x29=0.29))
    v30 = bx.run(pts, replace(bx.BEST, x29=0.3))
    xres = engine(x29)
    differ = [(r, e) for r, p, q, e in zip(x29, v29, v30, xres) if p[0] != q[0]]
    exact = sum(e[0] == r["excel"] for r, e in differ)
    model = sum(q[0] == r["excel"] for r, p, q in zip(x29, v29, v30) if p[0] != q[0])
    print(f"\n## x29\nengine bit-exact on {exact} of the {len(differ)} differing cells (the 0.3 model: {model})")


if __name__ == "__main__":
    main()
