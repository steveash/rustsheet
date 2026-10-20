import sys, numpy as np, struct
sys.argv=[sys.argv[0], sys.argv[1]] + sys.argv[2:]
S=sys.argv[1]; cut=float(sys.argv[3]) if len(sys.argv)>3 else 0.0
sys.argv=[sys.argv[0], S, str(cut)]
exec(open(S+'/glref.py').read())
rows=[l.split() for l in open(S+'/gl.sample')]
x=np.array([float.fromhex(r[0]) for r in rows]); y=np.array([float.fromhex(r[1]) for r in rows]); reg=np.array([r[2] for r in rows])
m=gammaln(x)
for name in dict.fromkeys(reg):
    s=reg==name; print(f"{name:8s} n={s.sum()} exact={np.mean(m[s]==y[s]):.4f}")
bits=lambda f: format(struct.unpack('<Q',struct.pack('<d',f))[0],'016x')
with open(S+'/gl.fixture','w') as o:
    o.write("# T-2128 GAMMALN fixture: x excel model (f64 bits). 1,500 random distinct x per region (x<0.7, K3 [0.7,1.5], (1.5,2],\n")
    o.write("# (2,2.02), [2.02,3], (3,8), x>=8) from the T-1941 kmyst-gam-* GAMMALN.PRECISE cells; model = the research checkers composed (glref.py).\n")
    for xi,yi,mi in zip(x,y,m): o.write(f"{bits(xi)} {bits(yi)} {bits(mi)}\n")
