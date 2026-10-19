# (7,7) rational minimax of Gamma on [lo,hi] in raw y, D0 fixed at -1/16; relative or absolute weight
# usage: python3 remez.py LO HI rel|abs START.txt OUT.txt  (START: 18 hex coefs N0..N7,0,D0,D1..D7,0); writes OUT.txt and OUT.txt.mp
import mpmath as mp, numpy as np, sys, math
mp.mp.dps=60
lo=mp.mpf(sys.argv[1]); hi=mp.mpf(sys.argv[2]); REL=sys.argv[3]=='rel'
x0=[float.fromhex(h) for h in open(sys.argv[4]).read().split()]
c=[mp.mpf(v) for v in x0[:8]]+[mp.mpf(v) for v in x0[10:17]]     # N0..N7, D1..D7
D0=mp.mpf(-1)/16
def R(c,t):
    N=mp.mpf(0); D=mp.mpf(0)
    for v in reversed(c[:8]): N=N*t+v
    for v in reversed(c[8:]): D=D*t+v
    D=D*t+D0; return N/D
def err(c,t):
    g=mp.gamma(t); return (R(c,t)-g)/(g if REL else 1)
def extrema(c,n=3000):
    ts=[lo+(hi-lo)*(1-mp.cos(mp.pi*k/(n-1)))/2 for k in range(n)]; es=[err(c,t) for t in ts]
    idx=[0]+[k for k in range(1,n-1) if (es[k]-es[k-1])*(es[k+1]-es[k])<=0]+[n-1]
    # merge into alternating sequence keeping largest |e| per sign run
    alt=[]
    for k in idx:
        if alt and mp.sign(es[k])==mp.sign(es[alt[-1]]):
            if abs(es[k])>abs(es[alt[-1]]): alt[-1]=k
        else: alt.append(k)
    while len(alt)>16:
        # drop the smaller end
        if abs(es[alt[0]])<abs(es[alt[-1]]): alt.pop(0)
        else: alt.pop()
    return [ts[k] for k in alt],[es[k] for k in alt],max(abs(e) for e in es)
pts,es,emax=extrema(c); print('start: alternants %d max err %s'%(len(pts),mp.nstr(emax,5)),flush=True)
if len(pts)<16:
    pts=[lo+(hi-lo)*(1-mp.cos(mp.pi*k/15))/2 for k in range(16)]
for it in range(25):
    # Newton on F_i = w_i (R(t_i)-f_i) - (-1)^i E = 0 ; unknowns c (15) + E
    u=list(c)+[mp.mpf(0)]
    for nt in range(30):
        F=mp.matrix([err(u[:15],t)-(-1)**i*u[15] for i,t in enumerate(pts)])
        J=mp.matrix(16,16)
        for j in range(16):
            h=(abs(u[j])+mp.mpf(1))*mp.mpf(10)**-25
            v=list(u); v[j]+=h
            Fj=[err(v[:15],t)-(-1)**i*v[15] for i,t in enumerate(pts)]
            for i in range(16): J[i,j]=(Fj[i]-F[i])/h
        d=mp.lu_solve(J,-F); u=[a+b for a,b in zip(u,d)]
        if mp.norm(d[:15])<mp.mpf(10)**-45*mp.norm(mp.matrix(u[:15])): break
    c=u[:15]; E=u[15]
    pts2,es,emax=extrema(c)
    print('it %d  E %s  max %s  ratio %s  alternants %d'%(it,mp.nstr(E,6),mp.nstr(emax,6),mp.nstr(emax/abs(E),8),len(pts2)),flush=True)
    if len(pts2)==16: pts=pts2
    if abs(emax/abs(E)-1)<mp.mpf(10)**-6: break
full=[float(v) for v in c[:8]]+[0.0,float(D0)]+[float(v) for v in c[8:]]+[0.0]
open(sys.argv[5],'w').write(' '.join(v.hex() for v in full)+'\n')
open(sys.argv[5]+'.mp','w').write(' '.join(mp.nstr(v,55) for v in c)+'\n')
ul=[(full[i]-x0[i])/math.ulp(x0[i]) if x0[i] else 0 for i in range(18)]
print('round53(R*) - start, ulps:',' '.join('%.3g'%v for v in ul))
