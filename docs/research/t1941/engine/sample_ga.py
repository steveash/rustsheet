"""Usage: sample_ga.py ga.u2 ga.sample K. Reservoir-samples K distinct x per GAMMA region from the sorted rows
(x_hex y_hex case) and writes x_hex y_hex region lines."""
import random, sys, math
random.seed(2129)
def reg(x):
    if x == math.floor(x): return "int"
    if x == math.floor(x) + 0.5: return "half"
    if x >= 10: return "big"
    if x >= 1: return "1to10"
    if x > 0: return "0to1"
    if -x <= 10: return "m10to0"
    return "refl"
K=int(sys.argv[3]); res={}; seen={}
for l in open(sys.argv[1]):
    a,b,_=l.split(); x=float.fromhex(a); r=reg(x)
    seen[r]=seen.get(r,0)+1; L=res.setdefault(r,[])
    if len(L)<K: L.append((a,b))
    else:
        j=random.randrange(seen[r])
        if j<K: L[j]=(a,b)
with open(sys.argv[2],"w") as o:
    for r in res:
        for a,b in res[r]: o.write(f"{a} {b} {r}\n")
print(seen)
