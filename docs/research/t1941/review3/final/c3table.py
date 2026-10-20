# Table hypothesis: is c3 a function of floor(x*2^k) (or floor((x-2)*2^k))? For pairs of informative runs with small dx,
# P(same c3 | same cell) vs P(same c3 | different cell), for k=8..16.
import numpy as np, pickle
rows=pickle.load(open('SCRATCH/c3all.pkl','rb'))
R=np.array([(r[0],r[1],r[2],r[3],r[4]) for r in rows]); x=R[:,0]; c3=R[:,2]; inf=(R[:,4]-R[:,3])>=0.01
xi=x[inf]; ci=c3[inf]; o=np.argsort(xi); xi=xi[o]; ci=ci[o]
pairs=[(i,j) for i in range(len(xi)) for j in range(i+1,min(i+6,len(xi))) if xi[j]-xi[i]<2e-3]
I=np.array([p[0] for p in pairs]); J=np.array([p[1] for p in pairs]); dx=xi[J]-xi[I]; same=np.abs(((ci[J]-ci[I])+0.5)%1-0.5)<=1/16+1e-9
print(f"{len(pairs)} informative pairs with dx<2e-3; P(same c3 within 1/16) overall {same.mean():.3f}")
for base,name in ((0.0,'x'),(2.0,'x-2')):
    for k in range(8,17):
        cell_i=np.floor((xi[I]-base)*2**k); cell_j=np.floor((xi[J]-base)*2**k); sc=cell_i==cell_j
        if sc.sum()>=5 and (~sc).sum()>=5:
            print(f"   {name} grid 2^-{k} ({2.0**-k:.1e}): same-cell pairs {sc.sum():4d} P(same c3) {same[sc].mean():.3f} | different-cell pairs {(~sc).sum():4d} P(same c3) {same[~sc].mean():.3f}")
for lo,hi in ((0,5e-5),(5e-5,1e-4),(1e-4,2e-4),(2e-4,3e-4),(3e-4,5e-4),(5e-4,1e-3),(1e-3,2e-3)):
    k=(dx>=lo)&(dx<hi)
    if k.sum(): print(f"   dx in [{lo:.0e},{hi:.0e}): n={k.sum():4d} P(same c3) {same[k].mean():.3f}")
print("dense pairs (dx<3e-4): x_i, x_j, c3_i, c3_j")
for i,j in zip(I[dx<3e-4][:40],J[dx<3e-4][:40]): print(f"   {xi[i]:.6f} {xi[j]:.6f}  {ci[i]:+.3f} {ci[j]:+.3f}  dx={xi[j]-xi[i]:.1e}")
