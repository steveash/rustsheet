"""Positive control for the frozen-model scorer: plant each model as synthetic 'Excel' values on probe a's BIN rows
(every 4th row) and score every model on them.  The diagonal must be 100% and the off-diagonal shows how far
apart the candidates are on these inputs (so a real-data difference of that size is resolvable)."""
import blib as B

rows = [r for r in B.load_probe("a") if r["fn"] == "BIN"][::4]
names = ["L-ext/LF0", "L-ext/LF1", "L-lcx/LF0", "L-lcx/LF1", "L-lcx/lnfR", "L-lcxx/LF0"]
vals = {m: [B.model_value(m, "BIN", r["k"], r["n"], r["p"]) for r in rows] for m in names}
print(f"{len(rows)} rows; planted (row) vs scored (column), exact matches")
print(" " * 12 + "".join(f"{m:>12s}" for m in names))
for t in names:
    print(f"{t:12s}" + "".join(f"{sum(a == b for a, b in zip(vals[t], vals[m])):12d}" for m in names))
