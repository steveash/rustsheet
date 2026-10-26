"""Scores (vlib's scorer) of round 3's ratio model (r3-fdensity c1_model ratio-xl-e) and the worker's brcomp/x
(fblib.model, gamln toms) on given sets, imported as models only. Usage: baselines.py SET ..."""
import sys

import vlib as V

sys.path.insert(0, str(V.T / "r4-f-brcomp"))
sys.path.insert(0, str(V.T / "r3-fdensity"))
import c1_model as C1  # noqa: E402
import fblib as FB  # noqa: E402

for s in sys.argv[1:]:
    rows, _ = V.case_rows(s)
    xl = [r["excel"] for r in rows]
    r3 = [float(C1.value(r["x"], r["d1"], r["d2"], C1.VARIANTS["ratio-xl-e"])) for r in rows]
    FB.prefetch_gl(rows, [{"gamln": "toms"}])
    bc = [float(FB.model(r["x"], r["d1"], r["d2"], {"gamln": "toms"})) for r in rows]
    print(f"# {s}: {len(rows)}  r3-ratio {V.fmt(V.tally(zip(r3, xl)))}  brcomp {V.fmt(V.tally(zip(bc, xl)))}")
