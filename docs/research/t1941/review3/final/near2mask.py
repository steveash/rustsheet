# Near-2 ladders as a structure test with grid_a (0.9953 stored): single-op masks (one product or one sum kept extended),
# pure double, and FMA at single stages.
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
Xs=[];Ys=[]
for c in ['e501','e502','e503','e504','e511','e512','r204','e401','e402','e403']:
    X,Y=load(c); m=(X>2)&(X<=3); Xs.append(X[m]); Ys.append(Y[m])
X=np.concatenate(Xs); YC=np.concatenate(Ys); XC=X.astype(LD); TC=sd(XC-LD(2)); T64=X-2.0
def hm(c,x,unP=set(),unS=set()):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        p=s*x
        if i not in unP: p=sd(p)
        s=p+LD(c[i])
        if i not in unS: s=sd(s)
    return s
def m(y): return float(np.mean(np.asarray(y,dtype=np.float64)==YC))
base_N=hm(P0,XC); base_D=hm(Q0,XC)
print(f"grid_a stored (x87 double-rounding sim): {m(sd(TC*sd(base_N/base_D))):.4f}")
def h64(c,x):
    s=np.full_like(x,c[0])
    for i in range(1,len(c)): s=s*x+c[i]
    return s
print(f"pure double (SSE2):                      {m(T64*(h64(P0,X)/h64(Q0,X))):.4f}")
print("single op kept extended (unstored), N side then D side:")
for i in range(1,10):
    print(f"   N.P{i} ext {m(sd(TC*sd(hm(P0,XC,unP={i})/base_D))):.4f}   N.s{i} ext {m(sd(TC*sd(hm(P0,XC,unS={i})/base_D))):.4f}")
for i in range(1,9):
    print(f"   D.P{i} ext {m(sd(TC*sd(base_N/hm(Q0,XC,unP={i})))):.4f}   D.d{i} ext {m(sd(TC*sd(base_N/hm(Q0,XC,unS={i})))):.4f}")
# FMA at a single stage (pure double otherwise)
def hfma(c,x,st):
    out=np.empty(len(x))
    for j,xv in enumerate(x):
        s=c[0]
        for i in range(1,len(c)): s=math.fma(s,xv,c[i]) if i==st else s*xv+c[i]
        out[j]=s
    return out
N64=h64(P0,X); D64=h64(Q0,X)
for st in range(1,10): print(f"   FMA at N stage {st}: {m(T64*(hfma(P0,X,st)/D64)):.4f}", end='')
print()
for st in range(1,9): print(f"   FMA at D stage {st}: {m(T64*(N64/hfma(Q0,X,st))):.4f}", end='')
print()
