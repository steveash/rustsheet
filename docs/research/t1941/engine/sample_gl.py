import random, sys
random.seed(2128)
regions=[("lt0.7",0,0.7),("k3",0.7,1.5),("k15",1.5,2.0000001),("k2near",2.0000001,2.02),("k2far",2.02,3),("red",3,8),("stir",8,1e309)]
K=int(sys.argv[3]); res={r[0]:[] for r in regions}; seen={r[0]:0 for r in regions}
lo07=float.fromhex('0x1.6666666666666p-1')
for l in open(sys.argv[1]):
    a,b,_=l.split(); x=float.fromhex(a)
    for name,lo,hi in regions:
        if name=="lt0.7": ok = x<lo07
        elif name=="k3": ok = lo07<=x<=1.5
        elif name=="k15": ok = 1.5<x<=2.0
        elif name=="k2near": ok = 2.0<x<2.02
        elif name=="k2far": ok = 2.02<=x<=3.0
        elif name=="red": ok = 3.0<x<8.0
        else: ok = x>=8.0
        if ok:
            seen[name]+=1; n=seen[name]; L=res[name]
            if len(L)<K: L.append((a,b))
            else:
                j=random.randrange(n)
                if j<K: L[j]=(a,b)
            break
with open(sys.argv[2],"w") as o:
    for name in res:
        for a,b in res[name]: o.write(f"{a} {b} {name}\n")
print(seen)
