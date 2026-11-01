"""T-3643 R4.2: generate probes-numeric/t3643-r4-log-modulus (seed 3643; written after the modulus model froze).

Run: python3 docs/research/t3643/gensweep.py eval/suites/probes-numeric/cases/t3643-r4-log-modulus.case.json
"""
import json, random, math, sys
rnd = random.Random(3643)
def lit(v): return f"{v:.15g}".replace("e+","E+").replace("e-","E-")
pts=[]
for _ in range(100):  # log-uniform over both parts, both signs
    x = rnd.choice([-1,1])*10**rnd.uniform(-150,150)
    y = rnd.choice([-1,1])*10**rnd.uniform(-150,150)
    pts.append((x,y))
for _ in range(60):  # |z| within ~1e-13 of 1, any angle
    t = rnd.uniform(-math.pi, math.pi); d = rnd.choice([-1,1])*10**rnd.uniform(-16,-13)
    pts.append(((1+d)*math.cos(t),(1+d)*math.sin(t)))
for _ in range(20):  # one part near +-1, the other log-uniform small
    a = rnd.choice([-1,1])*(1+rnd.choice([-1,1])*10**rnd.uniform(-15,-13)); b = rnd.choice([-1,1])*10**rnd.uniform(-12,-6)
    pts.append((a,b) if rnd.random()<0.5 else (b,a))
cells={}
for i,(x,y) in enumerate(pts, start=1):
    xs, ys = lit(x), lit(y)
    for col,fn in (("A","IMLN"),("B","IMLOG10"),("C","IMLOG2")):
        cells[f"{col}{i}"] = f'={fn}(COMPLEX({xs},{ys}))'
case={"format":"rustsheet-case/v1",
 "description":"T-3643 R4.2 fresh sweep (seed 3643, generated after the logs' modulus model froze): IMLN (A), IMLOG10 (B), IMLOG2 (C) of COMPLEX(x, y) for 100 log-uniform points over both parts (1E-150..1E150, both signs), 60 points with |z| within 1E-13 of 1 at a uniform angle, and 20 with one part near ±1 and the other 1E-12..1E-6.",
 "tags":["M9","engineering","probe","T-3643"],
 "sheets":[{"name":"S","cells":cells}],"expected":{}}
json.dump(case, open(sys.argv[1],"w"), indent=1); open(sys.argv[1],"a").write("\n")
print(len(cells))
