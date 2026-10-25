"""Oracle probe t2464-r2-bratio-gammaln: Excel's GAMMALN at every argument the bratio port asked the engine model for
(gammaln_cache.pkl), to check the engine's GAMMALN model at exactly the kernel's arguments (lnB = GL(a) + (GL(b) -
GL(a+b)) inherits any 1-ulp model error as 8-16 units at shapes 4-8). Arguments are exact-double literals (w2's enc).
usage: make_probe_gl.py [--write]"""
import importlib.util
import json
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("w2probes", HERE.parent / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w2p)
NAME = "t2464-r2-bratio-gammaln"
args = sorted(pickle.loads((HERE / "gammaln_cache.pkl").read_bytes()))[:16384]
cells = {f"A{i + 1}": f"=GAMMALN({w2p.enc(v)})" for i, v in enumerate(args)}
case = {"format": "rustsheet-case/v1",
        "description": "T-2464 r2-bratio: GAMMALN at the arguments Excel's incomplete-beta kernel uses (shapes and shape sums "
                       "of the BETA.DIST/F/T/BINOM evidence), to check the engine's GAMMALN model there",
        "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
p = w2p.CASES / f"{NAME}.case.json"
print(len(cells), "cells", p, "exists" if p.exists() else "")
if "--write" in sys.argv:
    p.write_text(json.dumps(case, indent=1) + "\n")
    json.dump(args, open(HERE / f"{NAME}.args.json", "w"))  # cell A{i+1} <-> args[i]
