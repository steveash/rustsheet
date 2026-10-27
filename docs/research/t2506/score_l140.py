"""T-2506: score the engine's L140 bup+bpser kernel on r2-bratio's scoring sets (acceptance criteria 1 and 2).

Rows come from r2-bratio `score_lib.load`; each is run through the engine's K(a, b, x, y) by the research harness
op `BRATIO` (closed forms, then the ported branches, else today's kernel), which also reports bratio's dispatch and
whether a ported form computed the point. Only rows the engine
dispatches to L140 bup+bpser are scored. Usage (from the repo root, after `cargo test --lib --no-run` of the
engine): T2464_HARNESS=target/debug/deps/rustsheet_engine-<hash> python3 docs/research/t2506/score_l140.py
"""
import os
import struct
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "t2464" / "r2-bratio"))
from score_lib import load, tally  # noqa: E402

L140_BUP_BPSER = 10  # bratio::Branch::L140BupBpser as u64
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
            res.append((fl(w), fl(w1), int(br), ported == "1"))
        return res


def main() -> None:
    rows = load(sets=("be", "w2f", "w4p1", "ix", "x29"))
    res = engine(rows)
    groups = defaultdict(list)
    for r, (w, w1, br, ported) in zip(rows, res):
        if br != L140_BUP_BPSER:
            continue
        m = w if r["side"] == "lower" else w1
        groups[r["set"]].append((m, r["excel"], r["cr"]))
        groups[r["set"] + (" ported" if ported else " today's kernel")].append((m, r["excel"], r["cr"]))
    print("| set | n | engine ex/u4/w15 | CR ex/u4/w15 |\n|---|---|---|---|")
    allc = []
    for s in sorted(groups):
        c = groups.get(s, [])
        allc += c
        n, e, u, v = tally((m, x) for m, x, _ in c)
        _, ce, cu, cv = tally((cr, x) for _, x, cr in c)
        print(f"| {s} | {n} | {e}/{u}/{v} | {ce}/{cu}/{cv} |")
    for name, sel in (("be+w2f+ix", ("be", "w2f", "ix")), ("ported", tuple(g for g in groups if g.endswith(" ported")))):
        c = [t for s in sel for t in groups.get(s, [])]
        n, e, u, v = tally((m, x) for m, x, _ in c)
        _, ce, cu, cv = tally((cr, x) for _, x, cr in c)
        print(f"| {name} | {n} | {e}/{u}/{v} | {ce}/{cu}/{cv} |")


if __name__ == "__main__":
    main()
