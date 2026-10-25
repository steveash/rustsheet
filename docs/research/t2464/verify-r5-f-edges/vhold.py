"""verify-r5-f-edges: scores M-edge's frozen predictions (hold_pred_frozen.json, written before the fill) and
the CR baseline on the verifier's hold-out, by section (hold_kinds.json from mkhold.py)."""
import json
import math
from collections import defaultdict

import vparse
import vscore as V

CASE = "t2586-verify-r5-f-edges-hold"


def pv(s):
    return s.strip("'") if s.startswith("'") else float(s)


def main():
    rs = vparse.rows(CASE)
    pred = {int(k): pv(v) for k, v in json.load(open("hold_pred_frozen.json")).items()}
    kinds = {int(k): v for k, v in json.load(open("hold_kinds.json")).items()}
    ME = V.model_edge()
    live = {r["row"]: ME(r["x"], r["d1"], r["d2"]) for r in rs}
    changed = [r["row"] for r in rs if repr(live[r["row"]]) != repr(pred[r["row"]])]
    print(f"# rows whose frozen prediction used a pre-fix x (parser kept a subnormal product): {changed}")
    for k in changed:
        pred[k] = live[k]  # same frozen model, corrected input x only
    drift = sum(1 for r in rs if repr(ME(r["x"], r["d1"], r["d2"])) != repr(pred[r["row"]]))
    bad_echo = [r for r in rs if isinstance(r["aecho"], float) and r["aecho"] != 0 and r["aecho"] != r["x"]]
    print(f"# {CASE}: {len(rs)} rows; frozen-vs-live model drift {drift}; x echo mismatches {len(bad_echo)}")
    by = defaultdict(list)
    for r in rs:
        k = kinds[r["row"]]
        if k == "vrun" or k == "tail":
            m, n = math.trunc(r["d1"]), math.trunc(r["d2"])
            k += "-" + ("d1=1" if m == 1 else "d1=2" if m == 2 else "d2=1" if n == 1 else "d2=2")
        by[k].append(r)
    allr = rs
    tot = lambda v, f: V.fmt(V.tally([(f(r), r["excel"]) for r in v]))  # noqa: E731
    me = lambda r: pred[r["row"]]  # noqa: E731
    crf = lambda r: V.cr(r["x"], r["d1"], r["d2"])  # noqa: E731
    print(f"{'all':12s} M-edge {tot(allr, me):24s} CR {tot(allr, crf)}")
    for k in sorted(by):
        print(f"{k:12s} M-edge {tot(by[k], me):24s} CR {tot(by[k], crf)}")
    rules = [r for r in rs if kinds[r["row"]] in ("zero", "neg", "df", "dfok", "subx", "minnorm", "ovf", "ovftr")]
    print(f"{'rules(all)':12s} M-edge {tot(rules, me):24s} CR {tot(rules, crf)}")
    tw = [r for r in rs if r["twin"]]
    same = sum(1 for r in tw if repr(r["excel"]) == repr(r["twin"]["excel"]))
    print(f"twins B==D bitwise {same}/{len(tw)}")
    dec = [r for r in rs if isinstance(r["excel"], str) or r["excel"] == 0 or isinstance(me(r), str) or me(r) == 0]
    print(f"error/zero decisions M-edge {sum(V.grade(me(r), r['excel'])[0] for r in dec)}/{len(dec)}")
    for r in dec:
        if not V.grade(me(r), r["excel"])[0]:
            print("   miss", kinds[r["row"]], r["row"], r["fa"], r["d1"], r["d2"], "excel", r["excel"], "B", r["B"], "C", r["C"], "model", me(r))
    for r in rs:
        if kinds[r["row"]] in ("thresh", "hthr", "minnorm") and not V.grade(me(r), r["excel"])[2]:
            e, p = r["excel"], me(r)
            rel = (p / e - 1) if isinstance(e, float) and isinstance(p, float) and e else None
            print("   num-miss", kinds[r["row"]], r["d1"], r["d2"], r["x"], "excel", e, "model", p, "rel", rel)


if __name__ == "__main__":
    main()
